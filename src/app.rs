use crate::menu::{Action, Menu, Screen};
use glam::{vec3, Mat4, Vec2, Vec3};
use miniquad::{window, EventHandler, KeyCode, KeyMods, MouseButton};
use std::collections::HashSet;
use stride::{
    course::{Run, Surface, AMBER, SECTIONS, VIOLET},
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
    course_run: Option<Run>,
    course_best: Option<f32>,
    course_overview: bool,
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
            toast_text: "STRIDE 0.5 / NEW LIGHTING AND MATERIALS".into(),
            best: None,
            course_run: None,
            course_best: None,
            course_overview: args.iter().any(|s| s == "--course-overview"),
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
        if args.iter().any(|s| s == "--materials-view") {
            game.camera.distance = 5.5;
            game.camera.yaw = -0.6;
            game.camera.pitch = 0.30;
            game.camera.update(game.player.pos, &game.world, 1.);
            game.toast = 0.;
        }
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
        if let Some(section) = args
            .iter()
            .position(|s| s == "--course-view")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse::<usize>().ok())
        {
            game.start_course(Some(section.min(7)));
        }
        if game.course_overview {
            game.start_course(None);
            game.toast = 0.;
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
        if args.iter().any(|s| s == "--course-menu") {
            game.open(Screen::Course);
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
        self.menu.screen.is_some() || self.course_overview
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
    fn start_course(&mut self, practice: Option<usize>) {
        self.world.reset_platforms();
        self.course_run = Some(Run::new(&self.world.course, practice));
        let run = self.course_run.as_ref().unwrap();
        self.checkpoint = self.world.course.nodes[run.checkpoint].position(&self.world);
        self.player = Player::default();
        self.player.respawn(self.checkpoint + Vec3::Y * 0.08);
        self.camera = Camera::new(self.player.pos);
        let next = self.world.course.nodes[run.next].position(&self.world);
        let direction = next - self.checkpoint;
        self.camera.yaw = (-direction.x).atan2(-direction.z);
        self.player.facing = self.camera.yaw;
        self.camera.distance = 12.;
        self.camera.update(self.player.pos, &self.world, 1.);
        self.sync_render();
        self.dust.clear();
        self.step_distance = 0.;
        self.toast = 5.;
        self.toast_text = if let Some(s) = practice {
            format!("PRACTICE {} / {}", s + 1, SECTIONS[s].0)
        } else {
            "SKYWAY / EIGHT SECTIONS. REACH THE SUMMIT.".into()
        };
    }
    fn respawn(&mut self) {
        if let Some(run) = &mut self.course_run {
            if !run.finished {
                run.retry();
            }
            self.checkpoint = self.world.course.nodes[run.checkpoint].position(&self.world);
        }
        self.player.respawn(self.checkpoint + Vec3::Y * 0.08);
        self.camera = Camera::new(self.player.pos);
        if let Some(run) = &self.course_run {
            if let Some(node) = self.world.course.nodes.get(run.next) {
                let direction = node.position(&self.world) - self.checkpoint;
                self.camera.yaw = (-direction.x).atan2(-direction.z);
                self.player.facing = self.camera.yaw;
                self.camera.update(self.player.pos, &self.world, 1.);
            }
        }
        self.sync_render();
        self.dust.clear();
        self.step_distance = 0.;
        self.toast_text = "BACK AT YOUR CHECKPOINT / KEEP GOING".into();
        self.toast = 3.;
    }
    fn restart(&mut self) {
        if let Some(run) = &self.course_run {
            let practice = run.practice;
            self.start_course(practice);
            return;
        }
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
        for p in &mut self.world.platforms {
            p.previous_center = (p.solid.min + p.solid.max) * 0.5;
        }
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
            Action::Course => self.open(Screen::Course),
            Action::StartCourse => {
                self.start_course(None);
                self.resume();
            }
            Action::Practice(s) => {
                self.start_course(Some(s));
                self.resume();
            }
            Action::Playground => {
                self.course_run = None;
                self.restart();
                self.resume();
            }
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
        self.world.advance(FIXED_DT);
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
        if self.course_run.is_none()
            && self.world.course.nodes[0].reached(&self.world, &self.player)
        {
            // Walking up the entrance ramp starts a full run without a teleport.
            self.world.reset_platforms();
            self.course_run = Some(Run::new(&self.world.course, None));
            self.checkpoint = self.world.course.nodes[0].position(&self.world);
            self.toast_text = "SKYWAY / BOARD THE AMBER SHUTTLE".into();
            self.toast = 5.;
        }
        if let Some(run) = &mut self.course_run {
            if run.advance(&self.world, &self.player) {
                self.checkpoint = self.world.course.nodes[run.checkpoint].position(&self.world);
                self.audio.play(Cue::Beacon, self.settings.volume, 0.65);
                self.toast_text = format!(
                    "CHECKPOINT SAVED / {}",
                    SECTIONS[run.section(&self.world.course)].0
                );
                self.toast = 4.;
                if run.finished {
                    if run.practice.is_none() {
                        self.course_best =
                            Some(self.course_best.map_or(run.elapsed, |t| t.min(run.elapsed)));
                    }
                    self.toast_text = if run.practice.is_some() {
                        "SECTION COMPLETE / CHOOSE ANOTHER IN PAUSE".into()
                    } else {
                        "SKYWAY COMPLETE / YOU REACHED THE SUMMIT!".into()
                    };
                    self.toast = 8.;
                }
            }
        }
        let missed_course = self
            .course_run
            .as_ref()
            .is_some_and(|r| !r.finished && self.player.pos.y < self.checkpoint.y - 6.);
        if self.player.pos.y < -12. || self.player.crushed || missed_course {
            self.respawn();
        }
        if let Some(b) = self
            .world
            .beacons
            .get(self.collected)
            .filter(|_| self.course_run.is_none())
        {
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
            &format!("FPS {:.0} / SIM 120 HZ / V0.5", self.fps),
            1.,
            MINT,
        );
        let x = w - 288.;
        ui.rect(x, 24., 264., 110., INK);
        ui.text(
            x + 18.,
            40.,
            &self.course_run.as_ref().map_or_else(
                || {
                    format!(
                        "BEACONS {:02} / {:02}",
                        self.collected,
                        self.world.beacons.len()
                    )
                },
                |r| {
                    format!(
                        "SKYWAY {:02} / {:02}",
                        r.next.min(self.world.course.nodes.len()),
                        self.world.course.nodes.len()
                    )
                },
            ),
            2.,
            WHITE,
        );
        ui.text(
            x + 18.,
            76.,
            &format!(
                "TIME {:05.1} S",
                self.course_run.as_ref().map_or(self.elapsed, |r| r.elapsed)
            ),
            2.,
            MINT,
        );
        if let Some(best) = if self.course_run.is_some() {
            self.course_best
        } else {
            self.best
        } {
            ui.text(x + 18., 110., &format!("BEST {:.1} S", best), 1., MUTED);
        }
        for (i, b) in self.world.beacons.iter().enumerate() {
            if self.course_run.is_some() || i < self.collected {
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
        ui.rect(w - 442., h - 166., 418., 70., INK);
        if let Some(run) = &self.course_run {
            let section = run.section(&self.world.course);
            ui.text(
                w - 425.,
                h - 151.,
                &format!(
                    "{} {:02}/08 / {}",
                    if run.practice.is_some() {
                        "PRACTICE"
                    } else {
                        "SECTION"
                    },
                    section + 1,
                    SECTIONS[section].0
                ),
                1.2,
                VIOLET,
            );
            ui.text(
                w - 425.,
                h - 132.,
                if run.finished {
                    "COMPLETE / CHOOSE A NEW RUN IN PAUSE"
                } else {
                    SECTIONS[section].1
                },
                1.25,
                WHITE,
            );
            ui.text(
                w - 425.,
                h - 111.,
                &format!("RETRIES {} / AMBER MOVES / MINT IS FIXED", run.failures),
                1.,
                MUTED,
            );
            for i in run.next..(run.next + 3).min(self.world.course.nodes.len()) {
                let node = &self.world.course.nodes[i];
                let p = matrix * (node.position(&self.world) + Vec3::Y * 2.).extend(1.);
                if p.w > 0. {
                    let ndc = p.truncate() / p.w;
                    if ndc.x.abs() < 0.92 && ndc.y.abs() < 0.65 {
                        let (px, py) = ((ndc.x * 0.5 + 0.5) * w, (-ndc.y * 0.5 + 0.5) * h);
                        ui.rect(px - 22., py - 12., 44., 24., INK);
                        ui.text(
                            px - 14.,
                            py - 6.,
                            &format!("{:02}", i + 1),
                            1.5,
                            if i == run.next { AMBER } else { MUTED },
                        );
                    }
                }
            }
        } else {
            ui.text(
                w - 425.,
                h - 151.,
                "PLAYGROUND / SKYWAY IN PAUSE MENU",
                1.1,
                VIOLET,
            );
            let hint = self
                .world
                .beacons
                .get(self.collected)
                .map_or("COMPLETE / RESTART IN PAUSE", |b| b.name);
            ui.text(w - 425., h - 132., hint, 1.25, WHITE);
            ui.text(
                w - 425.,
                h - 111.,
                "OR WALK UP THE WIDE RAMP TO THE EAST",
                1.,
                MUTED,
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
                    "{} SPRINT   RIGHT STICK CAMERA   START MENU / COURSES",
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
            let moving =
                self.movement().length_squared() > 0. || self.jump_pending || self.dive_pending;
            if let Some(run) = &mut self.course_run {
                run.clock(elapsed, moving);
            } else if self.started && self.collected < self.world.beacons.len() {
                self.elapsed += elapsed;
            }
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
        let mut camera = self.camera.interpolated(&self.previous_camera, alpha);
        if self.course_overview {
            camera.eye = vec3(265., 145., 245.);
            camera.target = vec3(135., 18., 72.);
        }
        let player = self.player.interpolated(&self.previous_player, alpha);
        let matrix = camera.matrix(rw as f32 / rh as f32);

        let dynamic = build_dynamic(
            &self.world,
            &player,
            self.time,
            self.collected,
            (&self.dust, self.landing_squash),
            alpha,
            self.course_run.as_ref(),
        );
        self.renderer.shadows(&self.scenery, &dynamic, player.pos);
        self.renderer.begin(matrix, camera.eye);
        self.renderer.static_mesh(&self.scenery, matrix, camera.eye);
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
            if i == 0 {
                1.
            } else if world
                .course
                .nodes
                .iter()
                .any(|n| n.surface == Surface::Fixed(i))
            {
                3.
            } else {
                0.
            },
        );
        if i > 0 && s.max.x - s.min.x > 2. {
            // Pale caps and a thin mint edge identify walkable surfaces.
            m.cube(
                vec3(
                    (s.min.x + s.max.x) * 0.5,
                    s.max.y + 0.012,
                    (s.min.z + s.max.z) * 0.5,
                ),
                vec3(s.max.x - s.min.x - 0.24, 0.024, s.max.z - s.min.z - 0.24),
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
    for (i, node) in world.course.nodes.iter().enumerate() {
        if let Surface::Fixed(_) = node.surface {
            let p = node.position(world);
            let slab = node.solid(world);
            let color = if node.checkpoint { VIOLET } else { MINT };
            for z in [slab.min.z + 0.12, slab.max.z - 0.12] {
                m.cube(
                    vec3(p.x, slab.max.y + 0.055, z),
                    vec3(slab.max.x - slab.min.x, 0.025, 0.12),
                    color,
                    2.,
                );
            }
            if node.checkpoint {
                m.ring(p + Vec3::Y * 0.065, 1.4, 0.1, VIOLET);
            }
            if let Some(next) = world.course.nodes.get(i + 1) {
                arrow(&mut m, p + Vec3::Y * 0.075, next.position(world) - p, color);
            }
        }
    }
    // Motion rails are visual guides below the pads, never invisible obstacles.
    for p in &world.platforms {
        let count = (p.travel.length() / 1.2).ceil().max(1.) as usize;
        for i in 0..=count {
            let center = p.origin + p.travel * (i as f32 / count as f32) - Vec3::Y * 0.55;
            m.cube(center, vec3(0.12, 0.12, 0.12), AMBER, 2.);
        }
        for center in [p.origin, p.origin + p.travel] {
            m.ring(center - Vec3::Y * 0.55, 0.5, 0.055, AMBER);
        }
    }
    // Runway launch stripe, low dive arch and the summit finish marks.
    m.cube(vec3(228., 29.07, 96.5), vec3(7.6, 0.025, 0.25), AMBER, 2.);
    m.cube(vec3(228., 31.17, 118.), vec3(6.8, 0.025, 0.2), AMBER, 2.);
    for i in 0..8 {
        for j in 0..2 {
            m.cube(
                vec3(97.5 + i as f32, 44.07, 70. + j as f32),
                vec3(1., 0.025, 1.),
                if (i + j) % 2 == 0 { VIOLET } else { NAVY },
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
            vec3(x + 100., height * 0.5 - 4., -260. - ((i * 3) % 8) as f32),
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
    feedback: (&[Dust], f32),
    alpha: f32,
    run: Option<&Run>,
) -> Mesh {
    let mut m = Mesh::default();
    let (dust, squash) = feedback;
    for platform in &world.platforms {
        let center = platform.rendered_center(alpha);
        m.cube(center, platform.size, [0.36, 0.29, 0.20], 3.);
        let top = center + Vec3::Y * (platform.size.y * 0.5 + 0.025);
        m.cube(
            top,
            vec3(platform.size.x - 0.20, 0.04, platform.size.z - 0.20),
            AMBER,
            3.,
        );
        for x in [-1., 1.] {
            m.cube(
                top + vec3(x * (platform.size.x * 0.5 - 0.15), 0.045, 0.),
                vec3(0.18, 0.025, platform.size.z - 0.3),
                [1., 0.90, 0.57],
                2.,
            );
        }
        arrow(
            &mut m,
            top + Vec3::Y * 0.055,
            platform.travel,
            [0.25, 0.26, 0.22],
        );
    }
    let next = run.map_or(0, |r| r.next);
    for (i, node) in world.course.nodes.iter().enumerate() {
        if i == 0 || (i >= next && i < next + 3) {
            let mut pos = node.position(world);
            if let Surface::Moving(j) = node.surface {
                pos += world.platforms[j].rendered_center(alpha)
                    - (world.platforms[j].solid.min + world.platforms[j].solid.max) * 0.5;
            }
            let color = if node.checkpoint {
                VIOLET
            } else if i == next {
                AMBER
            } else {
                MINT
            };
            m.ring(pos + Vec3::Y * 0.08, 0.7, 0.08, color);
            let start = m.vertices.len();
            m.cube(Vec3::ZERO, Vec3::splat(0.32), color, 2.);
            m.transform_from(
                start,
                Mat4::from_translation(pos + Vec3::Y * 2.)
                    * Mat4::from_rotation_y(time)
                    * Mat4::from_rotation_z(0.7),
            );
        }
    }
    if let Some(y) = world.floor_height(p.pos) {
        let height = (p.pos.y - y).max(0.);

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
        3.,
    );
    m.cube(vec3(0., 1.03, 0.25), vec3(0.38, 0.48, 0.18), ORANGE, 3.);
    m.cube(
        vec3(0., 1.49, 0.),
        vec3(0.58, 0.38, 0.47),
        [0.94, 0.96, 0.91],
        3.,
    );
    m.cube(vec3(0., 1.50, -0.245), vec3(0.46, 0.19, 0.03), NAVY, 4.);
    m.cube(
        vec3(-0.10, 1.50, -0.265),
        vec3(0.12, 0.045, 0.015),
        MINT,
        2.,
    );
    m.cube(vec3(0.10, 1.50, -0.265), vec3(0.12, 0.045, 0.015), MINT, 2.);
    m.cube(vec3(0., 0.72, 0.), vec3(0.48, 0.12, 0.38), NAVY, 4.);
    let swing = if p.grounded {
        p.animation_phase.sin() * (p.speed() / 9.).min(1.) * 0.7
    } else {
        -0.5
    };
    for side in [-1., 1.] {
        let leg = m.vertices.len();
        m.cube(vec3(0., -0.23, 0.), vec3(0.19, 0.46, 0.20), NAVY, 4.);
        m.cube(vec3(0., -0.50, -0.07), vec3(0.23, 0.15, 0.35), MINT, 0.);
        m.transform_from(
            leg,
            Mat4::from_translation(vec3(side * 0.16, 0.65, 0.))
                * Mat4::from_rotation_x(swing * side),
        );
        let arm = m.vertices.len();
        m.cube(vec3(0., -0.2, 0.), vec3(0.16, 0.44, 0.18), CONCRETE, 3.);
        m.cube(vec3(0., -0.45, 0.), vec3(0.17, 0.13, 0.20), ORANGE, 3.);
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

fn arrow(m: &mut Mesh, p: Vec3, direction: Vec3, color: [f32; 3]) {
    let forward = vec3(direction.x, 0., direction.z).normalize_or_zero();
    if forward == Vec3::ZERO {
        m.ring(p, 0.35, 0.07, color);
        return;
    }
    let right = vec3(-forward.z, 0., forward.x);
    m.triangle(
        p + forward * 0.9,
        p - forward * 0.35 + right * 0.5,
        p - forward * 0.35 - right * 0.5,
        color,
        2.,
    );
}

#[cfg(test)]
mod graphics_budget_tests {
    use super::*;
    #[test]
    fn authored_scene_and_animated_course_fit_gpu_index_budget() {
        let world = World::default();
        let scene = build_scene(&world);
        assert!(
            scene.vertices.len() < 60000,
            "{} static vertices",
            scene.vertices.len()
        );
        assert!(scene
            .indices
            .iter()
            .all(|&i| (i as usize) < scene.vertices.len()));
        let player = Player::default();
        let dynamic = build_dynamic(&world, &player, 0., 0, (&[], 0.), 0.5, None);
        assert!(dynamic.vertices.len() < 60000 && dynamic.indices.len() < 120000);
        assert!(dynamic
            .indices
            .iter()
            .all(|&i| (i as usize) < dynamic.vertices.len()));
    }
}
