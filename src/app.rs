use crate::menu::{Action, Menu, Screen};
use glam::{vec3, Mat4, Vec2, Vec3};
use miniquad::{window, EventHandler, KeyCode, KeyMods, MouseButton};
use std::collections::HashSet;
use stride::{
    engine::{
        audio::{Audio, Cue},
        camera::Camera,
        controller::Controller,
        frame::FramePacer,
        mesh::Mesh,
        physics::{Input, Move, Player},
        renderer::{viewport, GpuMesh, Renderer},
        ui::{Ui, INK, MUTED, WHITE},
        FIXED_DT,
    },
    settings::{Settings, BUTTON_NAMES},
    world::{World, CONCRETE, LEVEL_CENTER, LEVEL_SIZE, MINT, NAVY, ORANGE, SPAWN, TOWER},
};

pub struct Game {
    renderer: Renderer,
    scenery: GpuMesh,
    world: World,
    player: Player,
    previous_player: Player,
    previous_camera: Camera,
    audio: Audio,
    dust: Vec<Dust>,
    landing_squash: f32,
    step_distance: f32,
    camera: Camera,
    keys: HashSet<KeyCode>,
    controller: Controller,
    settings: Settings,
    menu: Menu,
    pacer: FramePacer,
    fps: f32,
    menu_direction: (i32, i32),
    menu_repeat: f32,
    jump_pending: bool,
    dive_pending: bool,
    orbit: bool,
    mouse: (f32, f32),
    accumulator: f32,
    time: f32,
    elapsed: f32,
    started: bool,
    collected: usize,
    checkpoint: Vec3,
    toast: f32,
    toast_text: String,
    best: Option<f32>,
    frames: u32,
    screenshot: Option<String>,
    smoke: bool,
    smoke_frames: u32,
}

impl Game {
    pub fn new(settings: Settings) -> Self {
        let world = World::default();
        let mut renderer = Renderer::new();
        renderer.resize(settings.size());
        let scenery = renderer.upload(&build_scene(&world));
        let args: Vec<String> = std::env::args().collect();
        let screenshot = args
            .iter()
            .position(|s| s == "--screenshot")
            .and_then(|i| args.get(i + 1))
            .cloned();
        let mut game = Self {
            renderer,
            scenery,
            world,
            player: Player::default(),
            previous_player: Player::default(),
            previous_camera: Camera::new(SPAWN),
            audio: Audio::new(!args.iter().any(|s| s == "--smoke-test")),
            dust: Vec::new(),
            landing_squash: 0.,
            step_distance: 0.,
            camera: Camera::new(SPAWN),
            keys: HashSet::new(),
            controller: Controller::default(),
            menu: Menu::new(settings.clone()),
            settings,
            pacer: FramePacer::default(),
            fps: 60.,
            menu_direction: (0, 0),
            menu_repeat: 0.,
            jump_pending: false,
            dive_pending: false,
            orbit: false,
            mouse: (0., 0.),
            accumulator: 0.,
            time: 0.,
            elapsed: 0.,
            started: false,
            collected: 0,
            checkpoint: SPAWN,
            toast: 7.,
            toast_text: "STRIDE 0.3 / DIVE. LAND. ROLL OUT.".into(),
            best: None,
            frames: 0,
            screenshot,
            smoke: args.iter().any(|s| s == "--smoke-test"),
            smoke_frames: args
                .iter()
                .position(|s| s == "--smoke-frames")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse().ok())
                .unwrap_or(10)
                .clamp(10, 3600),
        };
        if args.iter().any(|s| s == "--tower-view") {
            game.player.respawn(TOWER + vec3(9., 0.1, 28.));
            game.camera = Camera::new(game.player.pos);
            game.camera.yaw = 0.35;
            game.camera.pitch = 0.32;
            game.camera.distance = 17.;
            game.camera.update(game.player.pos, &game.world, 1.);
            game.collected = 5;
            game.toast = 0.;
        }
        if args.iter().any(|s| s == "--dive-view") {
            game.player.step(
                Input {
                    dive: true,
                    ..Default::default()
                },
                0.,
                &game.world,
                FIXED_DT,
            );
            for _ in 0..15 {
                game.player
                    .step(Input::default(), 0., &game.world, FIXED_DT);
            }
        }
        game.sync_render();
        if args.iter().any(|s| s == "--controller-options") {
            game.open(Screen::Controller);
        }
        if args.iter().any(|s| s == "--camera-options") {
            game.open(Screen::Camera);
        }
        if args.iter().any(|s| s == "--pause-view") {
            game.open(Screen::Pause);
        }
        if args.iter().any(|s| s == "--options") {
            game.open(Screen::Options);
        }
        if args.iter().any(|s| s == "--help-view") {
            game.open(Screen::Help);
        }
        game
    }
    fn down(&self, key: KeyCode) -> bool {
        self.keys.contains(&key)
    }
    fn paused(&self) -> bool {
        self.menu.screen.is_some()
    }
    fn movement(&self) -> Vec2 {
        let keyboard = Vec2::new(
            (self.down(KeyCode::D) || self.down(KeyCode::Right)) as i32 as f32
                - (self.down(KeyCode::A) || self.down(KeyCode::Left)) as i32 as f32,
            (self.down(KeyCode::W) || self.down(KeyCode::Up)) as i32 as f32
                - (self.down(KeyCode::S) || self.down(KeyCode::Down)) as i32 as f32,
        );
        if keyboard.length_squared() > 0. {
            keyboard.clamp_length_max(1.)
        } else {
            self.controller.state.movement
        }
    }
    fn sync_render(&mut self) {
        self.previous_player = self.player.clone();
        self.previous_camera = self.camera.clone();
    }
    fn respawn(&mut self) {
        self.player.respawn(self.checkpoint + Vec3::Y * 0.08);
        self.camera = Camera::new(self.player.pos);
        self.sync_render();
        self.dust.clear();
        self.step_distance = 0.;
        self.toast_text = "BACK AT YOUR CHECKPOINT / KEEP GOING".into();
        self.toast = 3.;
    }
    fn restart(&mut self) {
        self.player = Player::default();
        self.camera = Camera::new(SPAWN);
        self.sync_render();
        self.dust.clear();
        self.step_distance = 0.;
        self.collected = 0;
        self.elapsed = 0.;
        self.started = false;
        self.checkpoint = SPAWN;
        self.jump_pending = false;
        self.dive_pending = false;
        self.toast_text = "NEW RUN / SEVEN BEACONS. YOUR ROUTE.".into();
        self.toast = 4.;
    }
    fn clear_input(&mut self) {
        self.keys.clear();
        self.jump_pending = false;
        self.dive_pending = false;
        self.orbit = false;
        self.accumulator = 0.;
        self.menu_direction = (0, 0);
        self.menu_repeat = 0.;
        self.sync_render();
        self.controller.stop_rumble();
        self.audio.silence();
    }
    fn open(&mut self, screen: Screen) {
        self.clear_input();
        self.menu.open(screen, &self.settings);
    }
    fn resume(&mut self) {
        self.menu.screen = None;
        self.clear_input();
    }
    fn back(&mut self) {
        match self.menu.screen {
            Some(Screen::Pause) => self.resume(),
            Some(Screen::Camera | Screen::Controller | Screen::Audio) => {
                self.menu.open_sub(Screen::Options)
            }
            _ => self.open(Screen::Pause),
        }
    }
    fn action(&mut self, action: Action) {
        match action {
            Action::None => {}
            Action::Resume => self.resume(),
            Action::Options => self.open(Screen::Options),
            Action::Help => self.open(Screen::Help),
            Action::Camera => self.menu.open_sub(Screen::Camera),
            Action::Controller => self.menu.open_sub(Screen::Controller),
            Action::Audio => self.menu.open_sub(Screen::Audio),
            Action::Respawn => {
                self.respawn();
                self.resume();
            }
            Action::RequestRestart => self.open(Screen::ConfirmRestart),
            Action::Restart => {
                self.restart();
                self.resume();
            }
            Action::Quit => window::order_quit(),
            Action::Back => self.back(),
            Action::Apply => {
                self.settings = self.menu.draft.clone();
                self.controller.stop_rumble();
                self.audio.silence();
                self.renderer.resize(self.settings.size());
                window::set_fullscreen(self.settings.fullscreen);
                if !self.settings.fullscreen {
                    let (w, h) = self.settings.size();
                    window::set_window_size(w, h);
                }
                self.menu.status = if self.settings.save().is_ok() {
                    "APPLIED AND SAVED".into()
                } else {
                    "APPLIED / COULD NOT SAVE SETTINGS".into()
                };
            }
        }
    }
    fn confirm(&mut self) {
        let action = self.menu.confirm();
        self.action(action);
    }
    fn poll_controller(&mut self, dt: f32) {
        self.controller.poll(&self.settings);
        let pad = self.controller.state;
        if pad.pause {
            if self.paused() {
                self.resume();
            } else {
                self.open(Screen::Pause);
            }
            return;
        }
        if pad.help {
            if self.menu.screen == Some(Screen::Help) {
                self.resume();
            } else {
                self.open(Screen::Help);
            }
            return;
        }
        if self.paused() {
            if pad.back {
                self.back();
                return;
            }
            let direction = (
                if pad.menu_axis.x > 0.5 {
                    1
                } else if pad.menu_axis.x < -0.5 {
                    -1
                } else {
                    0
                },
                if pad.menu_axis.y > 0.5 {
                    -1
                } else if pad.menu_axis.y < -0.5 {
                    1
                } else {
                    0
                },
            );
            self.menu_repeat -= dt;
            if direction != (0, 0) && (direction != self.menu_direction || self.menu_repeat <= 0.) {
                if direction.1 != 0 {
                    self.menu.navigate(direction.1);
                } else {
                    self.menu.adjust(direction.0);
                }
                self.menu_repeat = if direction != self.menu_direction {
                    0.32
                } else {
                    0.16
                };
            }
            self.menu_direction = direction;
            if pad.confirm {
                self.confirm();
            }
        } else {
            self.jump_pending |= pad.jump;
            self.dive_pending |= pad.dive;
            if pad.recenter {
                self.camera.recenter(self.player.facing);
            }
        }
    }
    fn tick(&mut self) {
        self.sync_render();
        let movement = self.movement();
        if movement.length_squared() > 0. && !self.started {
            self.started = true;
        }
        if self.started && self.collected < self.world.beacons.len() {
            self.elapsed += FIXED_DT;
        }
        let turn = self.down(KeyCode::Q) as i32 - self.down(KeyCode::E) as i32;
        let look = self.controller.state.camera;
        if turn != 0 || look.length_squared() > 0. {
            let invert = if self.settings.invert_y { -1. } else { 1. };
            self.camera.orbit(
                (turn as f32 * 1.8 - look.x * 2.6) * FIXED_DT * self.settings.sensitivity(),
                -look.y * FIXED_DT * 1.8 * self.settings.sensitivity() * invert,
            );
        }
        self.player.step(
            Input {
                movement,
                sprint: self.down(KeyCode::LeftShift)
                    || self.down(KeyCode::RightShift)
                    || self.controller.state.sprint,
                jump: std::mem::take(&mut self.jump_pending),
                jump_held: self.down(KeyCode::Space) || self.controller.state.jump_held,
                dive: std::mem::take(&mut self.dive_pending),
            },
            self.camera.yaw,
            &self.world,
            FIXED_DT,
        );
        self.camera.follow(
            &self.player,
            self.settings.auto_camera,
            &self.world,
            FIXED_DT,
        );
        self.feedback();
        if self.player.pos.y < -12. {
            self.respawn();
        }
        if let Some(b) = self.world.beacons.get(self.collected) {
            let difference = self.player.pos - b.pos;
            if Vec2::new(difference.x, difference.z).length() < 1.25 && difference.y.abs() < 1.3 {
                self.audio.play(Cue::Beacon, self.settings.volume, 0.65);
                self.checkpoint = b.pos;
                self.collected += 1;
                self.toast = 4.;
                self.toast_text = format!("BEACON {} / CHECKPOINT SAVED", self.collected);
                if self.collected == self.world.beacons.len() {
                    self.best = Some(self.best.map_or(self.elapsed, |b| b.min(self.elapsed)));
                    self.toast_text = "COURSE COMPLETE / EXPLORE OR RESTART IN PAUSE".into();
                    self.toast = 8.;
                }
            }
        }
        self.time += FIXED_DT;
        self.toast = (self.toast - FIXED_DT).max(0.);
    }
    fn burst(&mut self, count: usize) {
        for i in 0..count {
            let a = i as f32 * 2.399;
            self.dust.push(Dust {
                pos: self.player.pos + Vec3::Y * 0.1,
                velocity: vec3(a.cos() * 1.8, 1. + (i % 3) as f32 * 0.25, a.sin() * 1.8),
                life: 0.3,
            });
        }
    }
    fn feedback(&mut self) {
        let prev = &self.previous_player;
        let cue = if self.player.wall_kicks != prev.wall_kicks {
            Some(Cue::Kick)
        } else if self.player.dives != prev.dives {
            Some(Cue::Dive)
        } else if self.player.rollouts != prev.rollouts {
            Some(Cue::Roll)
        } else if self.player.jumps != prev.jumps {
            Some(Cue::Jump)
        } else if self.player.landings != prev.landings {
            Some(Cue::Land)
        } else {
            None
        };
        if let Some(cue) = cue {
            self.audio.play(cue, self.settings.volume, 0.9);
            self.controller.rumble(self.settings.vibration, 9000, 18000);
            if matches!(cue, Cue::Land) {
                self.landing_squash = (self.player.impact_speed / 70.).clamp(0.04, 0.20);
                self.burst(10);
            } else {
                self.burst(5);
            }
        }
        if self.player.grounded && self.player.speed() > 1. && self.player.motion == Move::Normal {
            self.step_distance += self.player.speed() * FIXED_DT;
            if self.step_distance > 1.8 {
                self.step_distance -= 1.8;
                self.audio.play(Cue::Step, self.settings.volume, 0.45);
                self.burst(2);
            }
        } else {
            self.step_distance = 0.;
        }
        self.landing_squash = (self.landing_squash - FIXED_DT * 1.4).max(0.);
        for d in &mut self.dust {
            d.life -= FIXED_DT;
            d.pos += d.velocity * FIXED_DT;
            d.velocity.y -= 5. * FIXED_DT;
        }
        self.dust.retain(|d| d.life > 0.);
    }
    fn hud(&self, matrix: Mat4) -> Ui {
        let (w, h) = (1280., 720.);
        let mut ui = Ui::default();
        ui.rect(24., 24., 300., 110., INK);
        ui.rect(24., 24., 4., 110., MINT);
        ui.text(43., 40., "STRIDE", 4., WHITE);
        ui.text(44., 82., "RUST3D / MOVEMENT LAB", 1.5, MUTED);
        ui.text(
            44.,
            110.,
            &format!("FPS {:.0} / SIM 120 HZ / V0.3", self.fps),
            1.,
            MINT,
        );
        let x = w - 288.;
        ui.rect(x, 24., 264., 110., INK);
        ui.text(
            x + 18.,
            40.,
            &format!(
                "BEACONS {:02} / {:02}",
                self.collected,
                self.world.beacons.len()
            ),
            2.,
            WHITE,
        );
        ui.text(
            x + 18.,
            76.,
            &format!("TIME {:05.1} S", self.elapsed),
            2.,
            MINT,
        );
        if let Some(best) = self.best {
            ui.text(x + 18., 110., &format!("BEST {:.1} S", best), 1., MUTED);
        }
        for (i, b) in self.world.beacons.iter().enumerate() {
            if i < self.collected {
                continue;
            }
            let p = matrix * (b.pos + Vec3::Y * 2.5).extend(1.);
            if p.w <= 0. {
                continue;
            }
            let ndc = p.truncate() / p.w;
            if ndc.x.abs() > 0.92 || ndc.y.abs() > 0.6 {
                continue;
            }
            let px = (ndc.x * 0.5 + 0.5) * w;
            let py = (-ndc.y * 0.5 + 0.5) * h;
            let c = if i == self.collected { ORANGE } else { MUTED };
            ui.rect(px - 13., py - 13., 26., 26., INK);
            ui.text(px - 5., py - 7., &format!("{}", i + 1), 2., c);
        }
        ui.rect(24., h - 166., 252., 70., INK);
        ui.text(40., h - 151., self.player.action, 2., MINT);
        ui.text(
            40.,
            h - 124.,
            &format!("SPEED {:.1} M/S", self.player.speed()),
            1.5,
            WHITE,
        );
        ui.text(
            40.,
            h - 105.,
            if self.controller.name.is_some() {
                "CONTROLLER CONNECTED"
            } else {
                "KEYBOARD / MOUSE"
            },
            1.,
            MUTED,
        );
        let hint = self
            .world
            .beacons
            .get(self.collected)
            .map_or("COMPLETE / RESTART IN PAUSE", |b| b.name);
        ui.rect(w - 366., h - 166., 342., 70., INK);
        ui.text(w - 349., h - 151., "NEXT BEACON", 1., MUTED);
        ui.text(w - 349., h - 132., hint, 1.25, WHITE);
        if let Some(b) = self.world.beacons.get(self.collected) {
            ui.text(
                w - 349.,
                h - 111.,
                &format!("{:.0} M AWAY", (b.pos - self.player.pos).length()),
                1.,
                ORANGE,
            );
        }
        ui.rect(24., h - 80., w - 48., 56., INK);
        if self.controller.name.is_some() {
            ui.text(
                40.,
                h - 65.,
                &format!(
                    "LEFT STICK MOVE   {} JUMP   {} DIVE",
                    BUTTON_NAMES[self.settings.bindings[0]],
                    BUTTON_NAMES[self.settings.bindings[1]]
                ),
                1.5,
                WHITE,
            );
            ui.text(
                40.,
                h - 44.,
                &format!(
                    "{} SPRINT   RIGHT STICK CAMERA   START MENU / RESET",
                    BUTTON_NAMES[self.settings.bindings[2]]
                ),
                1.2,
                MUTED,
            );
        } else {
            ui.text(
                40.,
                h - 65.,
                "WASD MOVE   SPACE JUMP / ROLLOUT   F DIVE   SHIFT SPRINT",
                1.5,
                WHITE,
            );
            ui.text(
                40.,
                h - 44.,
                "Q/E OR RIGHT DRAG CAMERA   C RECENTER   F1 CONTROLS",
                1.2,
                MUTED,
            );
        }
        ui.text(w - 265., h - 45., "ESC PAUSE / F2 OPTIONS", 1., MINT);
        if self.toast > 0. && !self.paused() {
            let width = self.toast_text.len() as f32 * 7.2 + 32.;
            ui.rect((w - width) * 0.5, 145., width, 38., INK);
            ui.text((w - width) * 0.5 + 16., 159., &self.toast_text, 1.2, WHITE);
        }
        self.menu.draw(
            &mut ui,
            &self.settings,
            self.controller.name.as_deref(),
            self.fps,
            window::screen_size(),
        );
        ui
    }
    fn ui_position(&self, x: f32, y: f32) -> (f32, f32) {
        let (w, h) = window::screen_size();
        let (vx, vy, vw, vh) = viewport(w, h, self.settings.size());
        ((x - vx) * 1280. / vw, (y - vy) * 720. / vh)
    }
}

impl EventHandler for Game {
    fn update(&mut self) {
        let elapsed = self.pacer.begin(self.settings.cap());
        if self.frames > 0 && elapsed > 0.0001 {
            self.fps = if self.frames == 1 {
                1. / elapsed
            } else {
                self.fps * 0.9 + 0.1 / elapsed
            };
        }
        let dt = elapsed.min(0.1);
        self.poll_controller(dt);
        if !self.paused() {
            self.accumulator += dt;
            while self.accumulator >= FIXED_DT {
                self.tick();
                self.accumulator -= FIXED_DT;
            }
        }
    }
    fn draw(&mut self) {
        let (physical_w, physical_h) = window::screen_size();
        if physical_w <= 0. || physical_h <= 0. {
            return;
        }
        let (rw, rh) = self.renderer.size;
        let alpha = if self.paused() {
            1.
        } else {
            self.accumulator / FIXED_DT
        };
        let camera = self.camera.interpolated(&self.previous_camera, alpha);
        let player = self.player.interpolated(&self.previous_player, alpha);
        let matrix = camera.matrix(rw as f32 / rh as f32);
        self.renderer.begin();
        self.renderer.static_mesh(&self.scenery, matrix, camera.eye);
        let dynamic = build_dynamic(
            &self.world,
            &player,
            self.time,
            self.collected,
            &self.dust,
            self.landing_squash,
        );
        self.renderer.dynamic(&dynamic, matrix, camera.eye, false);
        let hud = self.hud(matrix);
        self.renderer.dynamic(
            &hud.mesh,
            Mat4::orthographic_rh_gl(0., 1280., 720., 0., -1., 1.),
            Vec3::ZERO,
            true,
        );
        self.frames += 1;
        self.renderer.finish(if self.frames == 8 {
            self.screenshot.as_deref()
        } else {
            None
        });
        if self.smoke && self.frames >= self.smoke_frames {
            println!(
                "SMOKE: {} frames / {:.1} FPS / render {}x{} / simulated {:.3}s / dives {} / rollouts {} / beacons {} / run {:.3}s",
                self.frames, self.fps, rw, rh, self.time,self.player.dives,self.player.rollouts,self.collected,self.elapsed
            );
            window::order_quit();
        }
    }
    fn key_down_event(&mut self, key: KeyCode, _mods: KeyMods, repeat: bool) {
        if key == KeyCode::Escape && !repeat {
            if self.paused() {
                self.back();
            } else {
                self.open(Screen::Pause);
            }
            return;
        }
        if key == KeyCode::F1 && !repeat {
            if self.menu.screen == Some(Screen::Help) {
                self.resume();
            } else {
                self.open(Screen::Help);
            }
            return;
        }
        if key == KeyCode::F2 && !repeat {
            self.open(Screen::Options);
            return;
        }
        if key == KeyCode::F11 && !repeat {
            self.settings.fullscreen = !self.settings.fullscreen;
            self.menu.draft.fullscreen = self.settings.fullscreen;
            window::set_fullscreen(self.settings.fullscreen);
            if !self.settings.fullscreen {
                let (w, h) = self.settings.size();
                window::set_window_size(w, h);
            }
            let _ = self.settings.save();
            return;
        }
        if self.paused() {
            match key {
                KeyCode::Up | KeyCode::W => self.menu.navigate(-1),
                KeyCode::Down | KeyCode::S => self.menu.navigate(1),
                KeyCode::Left | KeyCode::A => self.menu.adjust(-1),
                KeyCode::Right | KeyCode::D => self.menu.adjust(1),
                KeyCode::Enter | KeyCode::Space if !repeat => self.confirm(),
                _ => {}
            }
            return;
        }
        if repeat {
            return;
        }
        match key {
            KeyCode::F => self.dive_pending = true,
            KeyCode::C => self.camera.recenter(self.player.facing),
            KeyCode::Space => self.jump_pending = true,
            _ => {}
        }
        self.keys.insert(key);
    }
    fn key_up_event(&mut self, key: KeyCode, _mods: KeyMods) {
        self.keys.remove(&key);
    }
    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        if self.orbit && !self.paused() {
            let invert = if self.settings.invert_y { -1. } else { 1. };
            self.camera.orbit(
                -(x - self.mouse.0) * 0.006 * self.settings.sensitivity(),
                (y - self.mouse.1) * 0.004 * self.settings.sensitivity() * invert,
            );
        }
        if self.paused() {
            let (ux, uy) = self.ui_position(x, y);
            if let Some(row) = self.menu.row_at(ux, uy) {
                self.menu.selected = row;
            }
        }
        self.mouse = (x, y);
    }
    fn mouse_button_down_event(&mut self, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Right && !self.paused() {
            self.orbit = true;
            self.mouse = (x, y);
        }
        if button == MouseButton::Left && self.paused() {
            let (ux, uy) = self.ui_position(x, y);
            if let Some(row) = self.menu.row_at(ux, uy) {
                self.menu.selected = row;
                let editable = match self.menu.screen {
                    Some(Screen::Options | Screen::Camera) => row < 3,
                    Some(Screen::Controller) => row < 7,
                    Some(Screen::Audio) => row == 0,
                    _ => false,
                };
                if editable {
                    self.menu.adjust(if ux < 700. { -1 } else { 1 });
                } else {
                    self.confirm();
                }
            }
        }
    }
    fn mouse_button_up_event(&mut self, button: MouseButton, _x: f32, _y: f32) {
        if button == MouseButton::Right {
            self.orbit = false;
        }
    }
    fn mouse_wheel_event(&mut self, _x: f32, y: f32) {
        if !self.paused() {
            self.camera.distance -= y * 0.7;
        } else if y != 0. {
            self.menu.adjust(y.signum() as i32);
        }
    }
    fn window_minimized_event(&mut self) {
        if !self.paused() {
            self.open(Screen::Pause);
        }
    }
}

fn build_scene(world: &World) -> Mesh {
    let mut m = Mesh::default();
    for (i, s) in world.solids.iter().enumerate() {
        m.cube(
            (s.min + s.max) * 0.5,
            s.max - s.min,
            s.color,
            if i == 0 { 1. } else { 0. },
        );
        if i > 0 && s.max.x - s.min.x > 2. {
            // Pale caps and a thin mint edge identify walkable surfaces.
            m.cube(
                vec3(
                    (s.min.x + s.max.x) * 0.5,
                    s.max.y + 0.025,
                    (s.min.z + s.max.z) * 0.5,
                ),
                vec3(s.max.x - s.min.x, 0.05, s.max.z - s.min.z),
                [0.61, 0.69, 0.69],
                0.,
            );
            m.cube(
                vec3((s.min.x + s.max.x) * 0.5, s.max.y + 0.06, s.max.z - 0.06),
                vec3(s.max.x - s.min.x, 0.04, 0.12),
                MINT,
                2.,
            );
        }
    }
    for r in &world.ramps {
        let a = r.min;
        let b = r.max;
        let p = [
            vec3(a.x, a.y, b.z),
            vec3(b.x, a.y, b.z),
            vec3(b.x, b.y, a.z),
            vec3(a.x, b.y, a.z),
        ];
        m.quad(p, ORANGE, 0.);
        m.triangle(p[0], vec3(a.x, a.y, a.z), p[3], NAVY, 0.);
        m.triangle(p[1], p[2], vec3(b.x, a.y, a.z), NAVY, 0.);
        m.quad(
            [vec3(a.x, a.y, a.z), vec3(b.x, a.y, a.z), p[2], p[3]],
            NAVY,
            0.,
        );
        for z in 0..8 {
            let z = a.z + (b.z - a.z) * (z as f32 + 0.5) / 8.;
            let y = r.height((a.x + b.x) * 0.5, z).unwrap() + 0.012;
            let dy = (b.y - a.y) / (b.z - a.z) * 0.035;
            m.quad(
                [
                    vec3(a.x + 0.3, y + dy, z - 0.035),
                    vec3(a.x + 0.3, y - dy, z + 0.035),
                    vec3(b.x - 0.3, y - dy, z + 0.035),
                    vec3(b.x - 0.3, y + dy, z - 0.035),
                ],
                [0.95, 0.73, 0.51],
                2.,
            );
        }
    }
    // Starting pad and a guide stripe toward the first ramp.
    m.ring(SPAWN + Vec3::Y * 0.015, 1.5, 0.10, MINT);
    for i in 0..8 {
        m.cube(
            vec3(-i as f32, 0.015, 11.),
            vec3(0.42, 0.03, 0.12),
            MINT,
            2.,
        );
    }
    // Colour accents on the wall lane, and the long jump take-off edge.
    for x in [15.81, 18.59] {
        m.cube(vec3(x, 3., -3.), vec3(0.025, 0.10, 12.), ORANGE, 2.);
    }
    m.cube(vec3(2.44, 3.055, -16.), vec3(0.12, 0.025, 7.), ORANGE, 2.);
    for z in [-19., -17., -15., -13.] {
        m.cube(vec3(1.4, 3.06, z), vec3(0.8, 0.03, 0.25), ORANGE, 2.);
    }
    // Floating courtyard skirt and a distant architectural skyline.
    m.cube(
        LEVEL_CENTER + Vec3::Y * -0.7,
        vec3(LEVEL_SIZE.x - 1., 0.3, LEVEL_SIZE.z - 1.),
        NAVY,
        0.,
    );
    for i in 0..18 {
        let x = (i as f32 - 8.5) * 8.;
        let height = 6. + ((i * 7) % 13) as f32;
        m.cube(
            vec3(x, height * 0.5 - 4., -100. - ((i * 3) % 8) as f32),
            vec3(5., height, 5.),
            [0.49, 0.63, 0.69],
            0.,
        );
    }
    m
}

struct Dust {
    pos: Vec3,
    velocity: Vec3,
    life: f32,
}
fn build_dynamic(
    world: &World,
    p: &Player,
    time: f32,
    collected: usize,
    dust: &[Dust],
    squash: f32,
) -> Mesh {
    let mut m = Mesh::default();
    if let Some(y) = world.floor_height(p.pos) {
        let height = (p.pos.y - y).max(0.);
        let size = 0.5 + height.min(10.) * 0.025;
        m.disc(vec3(p.pos.x, y + 0.012, p.pos.z), size, [0.30, 0.39, 0.41]);
        if height > 0.3 {
            m.ring(vec3(p.pos.x, y + 0.025, p.pos.z), 0.36, 0.035, MINT);
        }
    }
    let start = m.vertices.len();
    // Courier: a little robot with a visor, backpack, and articulated legs.
    m.cube(
        vec3(0., 1.03, 0.),
        vec3(0.52, 0.64, 0.36),
        [0.9, 0.92, 0.83],
        0.,
    );
    m.cube(vec3(0., 1.03, 0.25), vec3(0.38, 0.48, 0.18), ORANGE, 0.);
    m.cube(
        vec3(0., 1.49, 0.),
        vec3(0.58, 0.38, 0.47),
        [0.94, 0.96, 0.91],
        0.,
    );
    m.cube(vec3(0., 1.50, -0.245), vec3(0.46, 0.19, 0.03), NAVY, 0.);
    m.cube(
        vec3(-0.10, 1.50, -0.265),
        vec3(0.12, 0.045, 0.015),
        MINT,
        2.,
    );
    m.cube(vec3(0.10, 1.50, -0.265), vec3(0.12, 0.045, 0.015), MINT, 2.);
    m.cube(vec3(0., 0.72, 0.), vec3(0.48, 0.12, 0.38), NAVY, 0.);
    let swing = if p.grounded {
        p.animation_phase.sin() * (p.speed() / 9.).min(1.) * 0.7
    } else {
        -0.5
    };
    for side in [-1., 1.] {
        let leg = m.vertices.len();
        m.cube(vec3(0., -0.23, 0.), vec3(0.19, 0.46, 0.20), NAVY, 0.);
        m.cube(vec3(0., -0.50, -0.07), vec3(0.23, 0.15, 0.35), MINT, 0.);
        m.transform_from(
            leg,
            Mat4::from_translation(vec3(side * 0.16, 0.65, 0.))
                * Mat4::from_rotation_x(swing * side),
        );
        let arm = m.vertices.len();
        m.cube(vec3(0., -0.2, 0.), vec3(0.16, 0.44, 0.18), CONCRETE, 0.);
        m.cube(vec3(0., -0.45, 0.), vec3(0.17, 0.13, 0.20), ORANGE, 0.);
        m.transform_from(
            arm,
            Mat4::from_translation(vec3(side * 0.37, 1.22, 0.))
                * Mat4::from_rotation_x(-swing * side),
        );
    }
    m.transform_from(
        start,
        Mat4::from_translation(p.pos + Vec3::Y * 0.02)
            * Mat4::from_rotation_y(p.facing)
            * match p.motion {
                Move::Dive | Move::Slide => {
                    Mat4::from_translation(Vec3::Y * 0.45)
                        * Mat4::from_rotation_x(-1.32)
                        * Mat4::from_translation(Vec3::Y * -0.8)
                }
                Move::Rollout => {
                    Mat4::from_translation(Vec3::Y * 0.8)
                        * Mat4::from_rotation_x(
                            -std::f32::consts::TAU * (p.motion_time / 0.45).min(1.),
                        )
                        * Mat4::from_translation(Vec3::Y * -0.8)
                }
                Move::Normal => {
                    Mat4::from_scale(vec3(1. + squash * 0.7, 1. - squash, 1. + squash * 0.7))
                }
            },
    );
    for d in dust {
        m.cube(
            d.pos,
            Vec3::splat(0.06 + d.life * 0.12),
            [0.73, 0.80, 0.75],
            0.,
        );
    }
    for (i, b) in world.beacons.iter().enumerate() {
        let c = if i < collected {
            MINT
        } else if i == collected {
            ORANGE
        } else {
            [0.60, 0.73, 0.76]
        };
        m.ring(b.pos + Vec3::Y * 0.075, 0.75, 0.09, c);
        let start = m.vertices.len();
        m.cube(Vec3::ZERO, Vec3::splat(0.44), c, 2.);
        m.transform_from(
            start,
            Mat4::from_translation(b.pos + Vec3::Y * (1.55 + (time * 2. + i as f32).sin() * 0.13))
                * Mat4::from_rotation_y(time * 1.2)
                * Mat4::from_rotation_z(0.7)
                * Mat4::from_rotation_x(0.6),
        );
        if i == collected {
            m.ring(
                b.pos + Vec3::Y * 0.08,
                0.93 + (time * 2.).sin() * 0.08,
                0.025,
                c,
            );
        }
    }
    m
}
