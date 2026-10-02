use glam::{vec3, Vec2, Vec3};
use stride::{
    engine::{
        controller::radial_deadzone,
        physics::{Input, Player, WALK_SPEED},
        renderer::viewport,
        FIXED_DT,
    },
    settings::{Settings, FRAME_CAPS, RESOLUTIONS},
    world::{Solid, World, NAVY, TOWER},
};

fn floor() -> World {
    World {
        terrain: None,
        solids: vec![Solid::new(vec3(0., -0.5, 0.), vec3(100., 1., 100.), NAVY)],
        ramps: vec![],
        beacons: vec![],
        platforms: vec![],
        course: Default::default(),
        time: 0.,
    }
}
fn player(pos: Vec3) -> Player {
    let mut p = Player::default();
    p.respawn(pos);
    p.grounded = true;
    p
}
fn input(x: f32, y: f32) -> Input {
    Input {
        movement: Vec2::new(x, y),
        ..Default::default()
    }
}
fn step(p: &mut Player, w: &World, i: Input, n: usize) {
    for _ in 0..n {
        p.step(i, 0., w, FIXED_DT);
    }
}

#[test]
fn ten_percent_deadzone_removes_idle_drift_on_both_axes() {
    for v in [
        Vec2::ZERO,
        Vec2::new(0.09, 0.),
        Vec2::new(0., -0.1),
        Vec2::new(0.06, 0.06),
        Vec2::new(f32::NAN, 0.),
    ] {
        assert_eq!(radial_deadzone(v, 0.1), Vec2::ZERO);
    }
    assert!(radial_deadzone(Vec2::new(0.1001, 0.), 0.1).length() < 0.001);
    assert!((radial_deadzone(Vec2::new(0.55, 0.), 0.1).x - 0.5).abs() < 0.001);
    assert!((radial_deadzone(Vec2::ONE, 0.1).length() - 1.).abs() < 0.001);
}
#[test]
fn analog_input_walks_and_zero_input_stops_quickly() {
    let w = floor();
    let mut p = player(Vec3::ZERO);
    step(&mut p, &w, input(0., 0.25), 120);
    assert!((p.speed() - WALK_SPEED * 0.25).abs() < 0.01);
    step(&mut p, &w, input(0., 1.), 120);
    let before = p.pos;
    step(&mut p, &w, Input::default(), 12);
    assert!(p.speed() < 0.001 && p.pos.distance(before) < 0.3);
}
#[test]
fn earned_speed_responds_smoothly_to_stick_strength_and_angle() {
    let w = floor();
    let after = |movement: Vec2, airborne: bool| {
        let mut p = player(vec3(0., if airborne { 20. } else { 0. }, 0.));
        p.grounded = !airborne;
        p.velocity = vec3(0., 0., -25.);
        step(&mut p, &w, input(movement.x, movement.y), 30);
        p.speed()
    };
    for airborne in [false, true] {
        let mut previous = after(Vec2::Y * 0.5, airborne);
        assert!(previous < 17., "light input should still brake: {previous}");
        for i in 1..=25 {
            let speed = after(Vec2::Y * (0.5 + i as f32 * 0.02), airborne);
            assert!(
                speed >= previous - 0.01 && speed - previous < 2.5,
                "stick cliff near {:.2}: {previous} -> {speed}",
                0.5 + i as f32 * 0.02
            );
            previous = speed;
        }
        assert!(
            previous > 24.5,
            "full input should keep momentum: {previous}"
        );
    }
    let mut previous = after(
        Vec2::new(70_f32.to_radians().sin(), 70_f32.to_radians().cos()),
        false,
    );
    for degrees in (72..=110).step_by(2) {
        let a = (degrees as f32).to_radians();
        let speed = after(Vec2::new(a.sin(), a.cos()), false);
        assert!(
            speed <= previous + 0.01 && previous - speed < 2.5,
            "turn cliff near {degrees} degrees: {previous} -> {speed}"
        );
        previous = speed;
    }
}
#[test]
fn reversal_and_visual_turn_are_responsive_but_continuous() {
    let w = floor();
    let mut p = player(Vec3::ZERO);
    step(&mut p, &w, input(0., 1.), 60);
    let facing = p.facing;
    p.step(input(0., -1.), 0., &w, FIXED_DT);
    assert!((p.facing - facing).abs() > 0. && (p.facing - facing).abs() < 0.2);
    step(&mut p, &w, input(0., -1.), 18);
    assert!(p.velocity.z > 6.);
}
#[test]
fn held_jump_is_higher_than_a_tap() {
    let w = floor();
    let mut high = player(Vec3::ZERO);
    let mut low = high.clone();
    for p in [&mut high, &mut low] {
        p.step(
            Input {
                jump: true,
                jump_held: true,
                ..Default::default()
            },
            0.,
            &w,
            FIXED_DT,
        );
    }
    let (mut high_apex, mut low_apex) = (0_f32, 0_f32);
    for _ in 0..120 {
        high.step(
            Input {
                jump_held: true,
                ..Default::default()
            },
            0.,
            &w,
            FIXED_DT,
        );
        low.step(Input::default(), 0., &w, FIXED_DT);
        high_apex = high_apex.max(high.pos.y);
        low_apex = low_apex.max(low.pos.y);
    }
    assert!(high_apex > 2.1 && low_apex < 1. && high_apex > low_apex * 2.);
    assert!(high.grounded && low.grounded);
}
fn wall() -> World {
    let mut w = floor();
    w.solids
        .push(Solid::new(vec3(2., 8., 0.), vec3(1., 16., 10.), NAVY));
    w
}
fn touch_wall(w: &World) -> Player {
    let mut p = player(vec3(1.17, 6., 0.));
    p.grounded = false;
    p.velocity = vec3(4., -9., 0.);
    p.step(input(1., 0.), 0., w, FIXED_DT);
    p
}
#[test]
fn wall_kick_requires_prompt_contact_and_wall_slide_is_brief() {
    let w = wall();
    let mut crisp = touch_wall(&w);
    crisp.step(
        Input {
            jump: true,
            ..Default::default()
        },
        0.,
        &w,
        FIXED_DT,
    );
    assert_eq!(crisp.wall_kicks, 1);
    assert!(crisp.velocity.x < -11. && crisp.velocity.y > 11.);
    let mut late = touch_wall(&w);
    step(&mut late, &w, input(1., 0.), 20);
    assert!(late.velocity.y < -6.);
    late.step(
        Input {
            jump: true,
            ..input(1., 0.)
        },
        0.,
        &w,
        FIXED_DT,
    );
    assert_eq!(
        late.wall_kicks, 0,
        "late wall attachment should not grant a kick"
    );
}
#[test]
fn wall_kick_grace_expires_after_leaving_the_wall() {
    let w = wall();
    let mut p = touch_wall(&w);
    p.velocity.x = -4.;
    step(&mut p, &w, Input::default(), 6);
    p.step(
        Input {
            jump: true,
            ..Default::default()
        },
        0.,
        &w,
        FIXED_DT,
    );
    assert_eq!(p.wall_kicks, 0);
}
#[test]
fn seven_beacon_level_is_larger_and_tower_can_be_climbed() {
    let w = World::default();
    assert_eq!(w.beacons.len(), 7);
    assert!(w.ramps.len() >= 6);
    assert!(w.solids[0].max.x - w.solids[0].min.x >= 112.);
    let points = [
        (0., 4.5),
        (4.5, 4.5),
        (4.5, 0.),
        (4.5, -4.5),
        (0., -4.5),
        (-4.5, -4.5),
        (-4.5, 0.),
        (-4.5, 4.5),
    ];
    let mut p = player(TOWER + vec3(0., 0., 7.));
    for i in 0..=13 {
        let goal = if i == 13 {
            TOWER + Vec3::Y * 14.3
        } else {
            let (x, z) = points[i % 8];
            TOWER + vec3(x, (i + 1) as f32 * 1.1, z)
        };
        let mut landed = false;
        for frame in 0..180 {
            let d = goal - p.pos;
            let direction = Vec2::new(d.x, -d.z).normalize_or_zero();
            let movement = if Vec2::new(d.x, d.z).length() > 0.12 {
                direction
            } else {
                Vec2::ZERO
            };
            p.step(
                Input {
                    movement,
                    jump: frame == 0 && i < 13,
                    jump_held: true,
                    ..Default::default()
                },
                0.,
                &w,
                FIXED_DT,
            );
            if frame > 5
                && p.grounded
                && (p.pos.y - goal.y).abs() < 0.01
                && Vec2::new(p.pos.x - goal.x, p.pos.z - goal.z).length() < 0.5
            {
                landed = true;
                break;
            }
        }
        assert!(landed, "ledge {i}, player {:?}, goal {goal:?}", p.pos);
    }
}
#[test]
fn simulation_is_consistent_at_30_60_and_144_render_fps() {
    let w = floor();
    let mut results = vec![];
    for fps in [30, 60, 144] {
        let mut p = player(Vec3::ZERO);
        let mut accumulator = 0_f64;
        for _ in 0..fps * 3 {
            accumulator += 1. / fps as f64;
            while accumulator + 1e-9 >= 1. / 120. {
                p.step(input(0., 1.), 0., &w, FIXED_DT);
                accumulator -= 1. / 120.;
            }
        }
        results.push(p.pos);
    }
    assert!(results[0].distance(results[1]) < 0.0001 && results[0].distance(results[2]) < 0.0001);
}
#[test]
fn settings_roundtrip_and_invalid_values_are_safe() {
    let s = Settings {
        resolution: 4,
        frame_cap: 7,
        fullscreen: true,
        deadzone_percent: 15,
        ..Default::default()
    };
    assert_eq!(Settings::decode(&s.encode()), s);
    assert_eq!(s.size(), (2560, 1440));
    assert_eq!(s.cap(), None);
    assert_eq!(
        Settings::decode("resolution=999\nframe_cap=-1\nfullscreen=oops\ndeadzone=101\n"),
        Settings::default()
    );
    assert_eq!(Settings::default().deadzone(), 0.1);
    for resolution in 0..RESOLUTIONS.len() {
        for frame_cap in 0..FRAME_CAPS.len() {
            let s = Settings {
                resolution,
                frame_cap,
                ..Default::default()
            };
            assert_eq!(Settings::decode(&s.encode()), s);
        }
    }
}
#[test]
fn render_resolution_preserves_aspect_at_other_window_sizes() {
    let (x, y, w, h) = viewport(1600., 1200., (1280, 720));
    assert_eq!((x, y, w, h), (0., 150., 1600., 900.));
    let (x, y, w, h) = viewport(2560., 1080., (1920, 1080));
    assert_eq!((x, y, w, h), (320., 0., 1920., 1080.));
}
