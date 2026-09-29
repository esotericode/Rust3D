use glam::{vec3, Mat4, Vec2, Vec3};
use miniquad::{date, window, EventHandler, KeyCode, KeyMods, MouseButton};
use std::collections::HashSet;
use stride::{
    engine::{
        camera::Camera,
        mesh::Mesh,
        physics::{Input, Player},
        renderer::{GpuMesh, Renderer},
        ui::{Ui, INK, MUTED, WHITE},
        FIXED_DT,
    },
    world::{World, CONCRETE, MINT, NAVY, ORANGE, SPAWN},
};

pub struct Game {
    renderer: Renderer,
    scenery: GpuMesh,
    world: World,
    player: Player,
    camera: Camera,
    keys: HashSet<KeyCode>,
    jump_pending: bool,
    orbit: bool,
    mouse: (f32, f32),
    paused: bool,
    help: bool,
    fullscreen: bool,
    previous: f64,
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
}

impl Game {
    pub fn new() -> Self {
        let world = World::default();
        let mut renderer = Renderer::new();
        let scenery = renderer.upload(&build_scene(&world));
        let args: Vec<String> = std::env::args().collect();
        let screenshot = args
            .iter()
            .position(|s| s == "--screenshot")
            .and_then(|i| args.get(i + 1))
            .cloned();
        Self {
            renderer,
            scenery,
            world,
            player: Player::default(),
            camera: Camera::new(SPAWN),
            keys: HashSet::new(),
            jump_pending: false,
            orbit: false,
            mouse: (0., 0.),
            paused: false,
            help: false,
            fullscreen: false,
            previous: date::now(),
            accumulator: 0.,
            time: 0.,
            elapsed: 0.,
            started: false,
            collected: 0,
            checkpoint: SPAWN,
            toast: 7.,
            toast_text: "WELCOME TO STRIDE / FOLLOW THE ORANGE BEACONS".into(),
            best: None,
            frames: 0,
            screenshot,
            smoke: args.iter().any(|s| s == "--smoke-test"),
        }
    }
    fn down(&self, key: KeyCode) -> bool {
        self.keys.contains(&key)
    }
    fn movement(&self) -> Vec2 {
        Vec2::new(
            (self.down(KeyCode::D) || self.down(KeyCode::Right)) as i32 as f32
                - (self.down(KeyCode::A) || self.down(KeyCode::Left)) as i32 as f32,
            (self.down(KeyCode::W) || self.down(KeyCode::Up)) as i32 as f32
                - (self.down(KeyCode::S) || self.down(KeyCode::Down)) as i32 as f32,
        )
    }
    fn respawn(&mut self) {
        self.player.respawn(self.checkpoint + Vec3::Y * 0.08);
        self.camera = Camera::new(self.player.pos);
        self.toast_text = "BACK AT YOUR CHECKPOINT / KEEP GOING".into();
        self.toast = 3.;
    }
    fn restart(&mut self) {
        self.player = Player::default();
        self.camera = Camera::new(SPAWN);
        self.collected = 0;
        self.elapsed = 0.;
        self.started = false;
        self.checkpoint = SPAWN;
        self.toast_text = "NEW RUN / FIVE BEACONS. YOUR ROUTE.".into();
        self.toast = 4.;
    }
    fn pause(&mut self) {
        self.paused = !self.paused;
        self.keys.clear();
        self.jump_pending = false;
        self.orbit = false;
        self.accumulator = 0.;
    }
    fn tick(&mut self) {
        let movement = self.movement();
        if movement.length_squared() > 0. && !self.started {
            self.started = true;
        }
        if self.started && self.collected < self.world.beacons.len() {
            self.elapsed += FIXED_DT;
        }
        let turn = self.down(KeyCode::Q) as i32 - self.down(KeyCode::E) as i32;
        self.camera.yaw += turn as f32 * FIXED_DT * 1.8;
        self.player.step(
            Input {
                movement,
                sprint: self.down(KeyCode::LeftShift) || self.down(KeyCode::RightShift),
                jump: std::mem::take(&mut self.jump_pending),
            },
            self.camera.yaw,
            &self.world,
            FIXED_DT,
        );
        self.camera.update(self.player.pos, &self.world, FIXED_DT);
        if self.player.pos.y < -12. {
            self.respawn();
        }
        if let Some(b) = self.world.beacons.get(self.collected) {
            let difference = self.player.pos - b.pos;
            if Vec2::new(difference.x, difference.z).length() < 1.25 && difference.y.abs() < 1.3 {
                self.checkpoint = b.pos;
                self.collected += 1;
                self.toast = 4.;
                self.toast_text = format!("BEACON {} / CHECKPOINT SAVED", self.collected);
                if self.collected == self.world.beacons.len() {
                    self.best = Some(self.best.map_or(self.elapsed, |b| b.min(self.elapsed)));
                    self.toast_text = "COURSE COMPLETE / EXPLORE, OR ENTER FOR ANOTHER RUN".into();
                    self.toast = 8.;
                }
            }
        }
        self.time += FIXED_DT;
        self.toast = (self.toast - FIXED_DT).max(0.);
    }

    fn hud(&self, w: f32, h: f32, matrix: Mat4) -> Ui {
        let mut ui = Ui::default();
        ui.rect(24., 24., 300., 94., INK);
        ui.rect(24., 24., 4., 94., MINT);
        ui.text(43., 40., "STRIDE", 4., WHITE);
        ui.text(44., 82., "RUST3D / MOVEMENT LAB", 1.5, MUTED);
        let x = w - 288.;
        ui.rect(x, 24., 264., 94., INK);
        ui.text(
            x + 18.,
            40.,
            &format!("BEACONS  {:02} / 05", self.collected),
            2.,
            WHITE,
        );
        ui.text(
            x + 18.,
            76.,
            &format!("TIME  {:05.1} S", self.elapsed),
            2.,
            MINT,
        );
        if let Some(best) = self.best {
            ui.text(x + 18., 104., &format!("BEST {:.1} S", best), 1., MUTED);
        }
        // Ordered beacon markers are projected from the actual 3D positions.
        for (i, b) in self.world.beacons.iter().enumerate() {
            if i < self.collected {
                continue;
            }
            let p = matrix * (b.pos + Vec3::Y * 2.5).extend(1.);
            if p.w <= 0. {
                continue;
            }
            let ndc = p.truncate() / p.w;
            if ndc.x.abs() > 0.92 || ndc.y.abs() > 0.7 {
                continue;
            }
            let px = (ndc.x * 0.5 + 0.5) * w;
            let py = (-ndc.y * 0.5 + 0.5) * h;
            let c = if i == self.collected { ORANGE } else { MUTED };
            ui.rect(px - 13., py - 13., 26., 26., INK);
            ui.text(px - 5., py - 7., &format!("{}", i + 1), 2., c);
        }
        // Compact movement telemetry.
        ui.rect(24., h - 166., 222., 70., INK);
        ui.text(40., h - 151., self.player.action, 2., MINT);
        ui.text(
            40.,
            h - 122.,
            &format!("SPEED {:.1} M/S", self.player.speed()),
            1.5,
            WHITE,
        );
        // Course hint.
        let hint = self
            .world
            .beacons
            .get(self.collected)
            .map_or("COMPLETE / ENTER TO RESTART", |b| b.name);
        ui.rect(24., h - 80., w - 48., 56., INK);
        ui.text(
            40.,
            h - 65.,
            "WASD MOVE   SPACE JUMP   SHIFT + SPACE LONG JUMP",
            1.5,
            WHITE,
        );
        ui.text(
            40.,
            h - 44.,
            "R RESPAWN   Q/E OR RIGHT DRAG CAMERA   F1 HELP",
            1.2,
            MUTED,
        );
        ui.text(w - 275., h - 45., "ESC PAUSE / F11 FULLSCREEN", 1., MINT);
        ui.rect(w - 366., h - 152., 342., 56., INK);
        ui.text(w - 349., h - 138., "NEXT BEACON", 1., MUTED);
        ui.text(w - 349., h - 119., hint, 1.25, WHITE);
        if self.toast > 0. {
            let width = self.toast_text.len() as f32 * 7.2 + 32.;
            ui.rect((w - width) * 0.5, 132., width, 38., INK);
            ui.text((w - width) * 0.5 + 16., 146., &self.toast_text, 1.2, WHITE);
        }
        if self.help || self.paused {
            let bx = (w - 620.) * 0.5;
            let by = (h - 436.) * 0.5;
            ui.rect(bx, by, 620., 436., INK);
            ui.rect(bx, by, 620., 4., MINT);
            ui.text(
                bx + 32.,
                by + 28.,
                if self.paused {
                    "TAKE A BREATHER"
                } else {
                    "LEARN THE MOVES"
                },
                3.,
                WHITE,
            );
            for (i, (key, description)) in [
                ("WASD / ARROWS", "MOVE RELATIVE TO THE CAMERA"),
                ("SHIFT", "SPRINT"),
                ("SPACE", "JUMP / WALL KICK IN THE AIR"),
                ("SHIFT + SPACE", "LONG JUMP WHILE RUNNING"),
                ("Q / E", "ORBIT CAMERA"),
                ("RIGHT DRAG", "LOOK AROUND / SCROLL TO ZOOM"),
                ("R / ENTER", "RESPAWN / RESTART THE COURSE"),
                ("F1 / F11", "HELP / FULLSCREEN"),
            ]
            .into_iter()
            .enumerate()
            {
                let y = by + 87. + i as f32 * 30.;
                ui.text(bx + 32., y, key, 1.5, MINT);
                ui.text(bx + 206., y, description, 1.4, WHITE);
            }
            ui.text(
                bx + 32.,
                by + 340.,
                "WALL KICK: JUMP INTO A WALL, THEN PRESS SPACE.",
                1.5,
                MUTED,
            );
            ui.text(
                bx + 32.,
                by + 362.,
                "ALTERNATE WALLS TO CLIMB. BEACONS SAVE YOUR SPOT.",
                1.5,
                MUTED,
            );
            if self.paused {
                ui.rect(bx + 32., by + 391., 260., 30., MINT);
                ui.text(bx + 70., by + 400., "ENTER / RESUME", 1.7, INK);
                ui.rect(bx + 328., by + 391., 260., 30., NAVY);
                ui.text(bx + 402., by + 400., "EXIT GAME", 1.7, WHITE);
            } else {
                ui.text(
                    bx + 32.,
                    by + 403.,
                    "F1 TO CLOSE / THE LAB IS YOURS",
                    1.5,
                    MINT,
                );
            }
        }
        ui
    }
}

impl EventHandler for Game {
    fn update(&mut self) {
        let now = date::now();
        let dt = ((now - self.previous) as f32).clamp(0., 0.05);
        self.previous = now;
        if !self.paused {
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
        let matrix = self.camera.matrix(physical_w / physical_h);
        self.renderer.begin();
        self.renderer
            .static_mesh(&self.scenery, matrix, self.camera.eye);
        let dynamic = build_dynamic(&self.world, &self.player, self.time, self.collected);
        self.renderer
            .dynamic(&dynamic, matrix, self.camera.eye, false);
        // Layout in logical coordinates scales consistently on high-DPI displays.
        let w = 1280.;
        let h = w * physical_h / physical_w;
        let hud = self.hud(w, h, matrix);
        self.renderer.dynamic(
            &hud.mesh,
            Mat4::orthographic_rh_gl(0., w, h, 0., -1., 1.),
            Vec3::ZERO,
            true,
        );
        self.frames += 1;
        if self.frames == 8 {
            if let Some(path) = &self.screenshot {
                self.renderer
                    .capture(path, physical_w as usize, physical_h as usize)
                    .expect("save screenshot");
            }
        }
        self.renderer.finish();
        if self.smoke && self.frames >= 10 {
            window::order_quit();
        }
    }
    fn key_down_event(&mut self, key: KeyCode, _mods: KeyMods, repeat: bool) {
        if repeat {
            return;
        }
        match key {
            KeyCode::Escape => self.pause(),
            KeyCode::F1 => self.help = !self.help,
            KeyCode::F11 => {
                self.fullscreen = !self.fullscreen;
                window::set_fullscreen(self.fullscreen);
            }
            KeyCode::Enter => {
                if self.paused {
                    self.pause();
                } else {
                    self.restart();
                }
            }
            KeyCode::R if !self.paused => self.respawn(),
            KeyCode::Space if !self.paused => {
                self.jump_pending = true;
            }
            _ => {}
        }
        if !self.paused {
            self.keys.insert(key);
        }
    }
    fn key_up_event(&mut self, key: KeyCode, _mods: KeyMods) {
        self.keys.remove(&key);
    }
    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        if self.orbit && !self.paused {
            self.camera.yaw -= (x - self.mouse.0) * 0.006;
            self.camera.pitch += (y - self.mouse.1) * 0.004;
        }
        self.mouse = (x, y);
    }
    fn mouse_button_down_event(&mut self, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Right {
            self.orbit = true;
            self.mouse = (x, y);
        }
        if button == MouseButton::Left && self.paused {
            let (pw, ph) = window::screen_size();
            let x = x * 1280. / pw;
            let y = y * 1280. / pw;
            let bx = (1280. - 620.) * 0.5;
            let by = (1280. * ph / pw - 436.) * 0.5;
            if y >= by + 391. && y <= by + 421. {
                if x >= bx + 32. && x <= bx + 292. {
                    self.pause();
                }
                if x >= bx + 328. && x <= bx + 588. {
                    window::order_quit();
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
        if !self.paused {
            self.camera.distance -= y * 0.7;
        }
    }
    fn window_minimized_event(&mut self) {
        if !self.paused {
            self.pause();
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
    m.cube(vec3(0., -1.3, -3.), vec3(47., 0.3, 45.), NAVY, 0.);
    for i in 0..18 {
        let x = (i as f32 - 8.5) * 8.;
        let height = 6. + ((i * 7) % 13) as f32;
        m.cube(
            vec3(x, height * 0.5 - 4., -54. - ((i * 3) % 8) as f32),
            vec3(5., height, 5.),
            [0.49, 0.63, 0.69],
            0.,
        );
    }
    m
}

fn build_dynamic(world: &World, p: &Player, time: f32, collected: usize) -> Mesh {
    let mut m = Mesh::default();
    if let Some(y) = world.floor_height(p.pos) {
        let size = 0.55 + (p.pos.y - y) * 0.02;
        m.disc(vec3(p.pos.x, y + 0.012, p.pos.z), size, [0.36, 0.44, 0.45]);
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
        (time * 13.).sin() * (p.speed() / 9.).min(1.) * 0.7
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
        Mat4::from_translation(p.pos + Vec3::Y * 0.02) * Mat4::from_rotation_y(p.facing),
    );
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
