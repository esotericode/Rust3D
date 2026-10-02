use glam::{vec3, Vec2, Vec3};
use stride::{
    engine::{
        camera::{Camera, MAX_VERTICAL_LAG},
        controller::map_buttons,
        physics::{Input, Move, Player, HEIGHT},
        FIXED_DT,
    },
    settings::Settings,
    world::{Solid, World, NAVY},
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
fn dive(p: &mut Player, w: &World) {
    p.step(
        Input {
            dive: true,
            movement: Vec2::Y,
            ..Default::default()
        },
        0.,
        w,
        FIXED_DT,
    );
}
fn slide(p: &mut Player, w: &World) {
    for _ in 0..120 {
        p.step(Input::default(), 0., w, FIXED_DT);
        if p.motion == Move::Slide {
            return;
        }
    }
    panic!("dive did not land: {p:?}");
}
#[test]
fn ground_dive_lands_and_rollout_keeps_momentum() {
    let w = floor();
    let mut p = player(Vec3::ZERO);
    dive(&mut p, &w);
    assert_eq!(p.motion, Move::Dive);
    assert!(p.speed() >= 13. && p.velocity.y > 4. && p.height() < HEIGHT);
    slide(&mut p, &w);
    assert!(p.grounded && p.speed() > 10. && p.dives == 1);
    let speed = p.speed();
    p.step(
        Input {
            jump: true,
            ..Default::default()
        },
        0.,
        &w,
        FIXED_DT,
    );
    assert_eq!(p.motion, Move::Rollout);
    assert!(p.velocity.y > 7. && p.speed() >= speed - 0.1 && p.rollouts == 1);
    for _ in 0..130 {
        p.step(Input::default(), 0., &w, FIXED_DT);
    }
    assert!(p.grounded && p.motion == Move::Normal);
}
#[test]
fn repeated_dive_presses_do_not_add_midair_boosts() {
    let w = floor();
    let mut p = player(vec3(0., 8., 0.));
    p.grounded = false;
    dive(&mut p, &w);
    for _ in 0..20 {
        p.step(
            Input {
                dive: true,
                ..Default::default()
            },
            0.,
            &w,
            FIXED_DT,
        );
    }
    assert_eq!(p.dives, 1);
    assert_eq!(p.motion, Move::Dive);
    assert!(p.velocity.y < 0.);
}
#[test]
fn jump_just_before_dive_landing_buffers_rollout() {
    let w = floor();
    let mut p = player(Vec3::ZERO);
    dive(&mut p, &w);
    for _ in 0..120 {
        let press = p.pos.y < 0.1 && p.velocity.y < 0. && p.motion == Move::Dive;
        p.step(
            Input {
                jump: press,
                ..Default::default()
            },
            0.,
            &w,
            FIXED_DT,
        );
        if p.rollouts > 0 {
            break;
        }
    }
    assert_eq!(p.rollouts, 1);
    assert!(p.velocity.y > 0.);
}
#[test]
fn rollout_that_lands_on_a_ledge_can_jump_immediately() {
    let mut w = floor();
    w.solids
        .push(Solid::new(vec3(0., 0.4, -6.), vec3(20., 0.8, 8.), NAVY));
    let mut p = player(vec3(0., 0., 2.));
    p.motion = Move::Slide;
    p.velocity = vec3(0., 0., -10.);
    p.step(
        Input {
            jump: true,
            ..Default::default()
        },
        0.,
        &w,
        FIXED_DT,
    );
    assert_eq!(p.motion, Move::Rollout);
    for _ in 0..80 {
        p.step(
            Input {
                movement: Vec2::Y,
                ..Default::default()
            },
            0.,
            &w,
            FIXED_DT,
        );
        if p.grounded {
            break;
        }
    }
    assert!(p.grounded && p.pos.y > 0.79, "{p:?}");
    assert_eq!(
        p.motion,
        Move::Normal,
        "early landing should end the rollout"
    );
    p.step(
        Input {
            jump: true,
            jump_held: true,
            movement: Vec2::Y,
            ..Default::default()
        },
        0.,
        &w,
        FIXED_DT,
    );
    assert!(p.velocity.y > 9. && p.jumps == 1, "{p:?}");
}
#[test]
fn slide_recovers_without_input_and_dive_can_be_used_again() {
    let w = floor();
    let mut p = player(Vec3::ZERO);
    dive(&mut p, &w);
    slide(&mut p, &w);
    for _ in 0..75 {
        p.step(Input::default(), 0., &w, FIXED_DT);
    }
    assert_eq!(p.motion, Move::Normal);
    dive(&mut p, &w);
    assert_eq!(p.dives, 2);
}
#[test]
fn slides_carve_gentle_turns_without_losing_extra_speed() {
    let w = floor();
    let mut p = player(Vec3::ZERO);
    p.motion = Move::Slide;
    p.velocity = vec3(0., 0., -20.);
    for _ in 0..60 {
        p.step(
            Input {
                movement: Vec2::X,
                sprint: true,
                ..Default::default()
            },
            0.,
            &w,
            FIXED_DT,
        );
    }
    assert_eq!(p.motion, Move::Slide);
    let turned = p.velocity.x.atan2(-p.velocity.z).to_degrees();
    assert!(
        turned > 15. && turned < 40.,
        "slide turned {turned} degrees"
    );
    assert!((p.speed() - (20. - 2.8 * 0.5)).abs() < 0.2, "{p:?}");
}
#[test]
fn slide_cannot_roll_out_through_a_low_ceiling_and_can_crawl_out() {
    let mut w = floor();
    let mut p = player(Vec3::ZERO);
    dive(&mut p, &w);
    slide(&mut p, &w);
    let roof = Solid::new(p.pos + vec3(0., 1.15, 0.), vec3(4., 0.4, 4.), NAVY);
    w.solids.push(roof);
    p.step(
        Input {
            jump: true,
            ..Default::default()
        },
        0.,
        &w,
        FIXED_DT,
    );
    assert_eq!(p.motion, Move::Slide);
    assert_eq!(p.rollouts, 0);
    for _ in 0..180 {
        p.step(
            Input {
                movement: Vec2::X,
                ..Default::default()
            },
            0.,
            &w,
            FIXED_DT,
        );
        assert!(p.pos.is_finite());
    }
    assert_eq!(p.motion, Move::Normal);
    assert!(p.pos.x > 2.3);
}
#[test]
fn capsule_passes_square_corner_clearance_and_reports_diagonal_normals() {
    let mut w = floor();
    w.solids
        .push(Solid::new(vec3(0., 3., 0.), vec3(2., 6., 2.), NAVY));
    let mut p = player(vec3(-1.24, 0., 1.24));
    p.velocity.x = 1.;
    p.step(
        Input {
            movement: Vec2::X,
            ..Default::default()
        },
        0.,
        &w,
        FIXED_DT,
    );
    assert!(p.pos.x > -1.24 && (p.pos.z - 1.24).abs() < 0.0001);
    let mut p = player(vec3(-1.22, 2., 1.22));
    p.grounded = false;
    p.velocity = vec3(3., 0., -3.);
    p.step(Input::default(), 0., &w, FIXED_DT);
    assert!(p.wall_normal.x < -0.5 && p.wall_normal.z > 0.5);
    assert!((p.wall_normal.length() - 1.).abs() < 0.001);
}
#[test]
fn leg_cadence_holds_steady_at_high_speed() {
    let w = floor();
    let footfalls_per_second = |speed: f32| {
        let mut p = player(Vec3::ZERO);
        p.velocity = vec3(0., 0., -speed);
        let start = p.animation_phase;
        for _ in 0..60 {
            p.step(
                Input {
                    movement: Vec2::Y,
                    sprint: true,
                    ..Default::default()
                },
                0.,
                &w,
                FIXED_DT,
            );
        }
        (p.animation_phase - start) / std::f32::consts::PI / 0.5
    };
    let sprint = footfalls_per_second(10.5);
    let fast = footfalls_per_second(50.);
    assert!(
        sprint > 5.5 && sprint < 7.,
        "{sprint} footfalls/s at sprint"
    );
    assert!(fast > sprint && fast < 9., "{fast} footfalls/s at 50 m/s");
}
#[test]
fn rendering_interpolates_without_changing_simulation_and_wraps_facing() {
    let previous = player(Vec3::ZERO);
    let mut current = previous.clone();
    current.pos = Vec3::X;
    current.facing = std::f32::consts::TAU - 0.1;
    let render = current.interpolated(&previous, 0.5);
    assert_eq!(render.pos, Vec3::X * 0.5);
    assert!((render.facing + 0.05).abs() < 0.001);
    assert_eq!(current.pos, Vec3::X);
}
#[test]
fn camera_retracts_immediately_and_recovers_smoothly() {
    let mut w = floor();
    let mut camera = Camera::new(Vec3::ZERO);
    w.solids
        .push(Solid::new(vec3(0., 3., 3.), vec3(5., 6., 1.), NAVY));
    camera.update(Vec3::ZERO, &w, FIXED_DT);
    assert!(camera.eye.z < 2.22 && camera.matrix(16. / 9.).is_finite());
    let short = camera.eye.distance(camera.target);
    w.solids.pop();
    camera.update(Vec3::ZERO, &w, FIXED_DT);
    assert!(
        camera.eye.distance(camera.target) > short
            && camera.eye.distance(camera.target) < short + 1.
    );
    for _ in 0..180 {
        camera.update(Vec3::ZERO, &w, FIXED_DT);
    }
    assert!(camera.eye.distance(camera.target) > 9.9);
}
#[test]
fn camera_keeps_a_fast_falling_player_in_frame() {
    let w = World {
        terrain: None,
        solids: vec![],
        ramps: vec![],
        beacons: vec![],
        platforms: vec![],
        course: Default::default(),
        time: 0.,
    };
    let mut p = player(vec3(0., 200., 0.));
    p.grounded = false;
    let mut camera = Camera::new(p.pos);
    for _ in 0..240 {
        p.step(Input::default(), 0., &w, FIXED_DT);
        camera.follow(&p, false, &w, FIXED_DT);
        let lag = camera.target.y - (p.pos.y + 1.1);
        assert!(lag <= MAX_VERTICAL_LAG + 0.001, "camera trails by {lag} m");
    }
    assert!(p.velocity.y < -60., "{p:?}");
}
#[test]
fn auto_camera_respects_manual_input_and_vertical_climbing_remains_finite() {
    let w = floor();
    let mut c = Camera::new(Vec3::ZERO);
    let mut p = player(vec3(0., 3., 0.));
    p.facing = 2.;
    p.velocity.x = 7.;
    c.orbit(0.5, 0.);
    for _ in 0..100 {
        c.follow(&p, true, &w, FIXED_DT);
    }
    assert!((c.yaw - 0.5).abs() < 0.001 && c.target.y > 3.9);
    for _ in 0..250 {
        p.pos.y = 10.;
        c.follow(&p, true, &w, FIXED_DT);
    }
    assert!(c.yaw > 1.8 && c.matrix(16. / 9.).is_finite());
}
#[test]
fn west_button_dives_and_remapping_does_not_change_menu_confirm() {
    let mut settings = Settings::default();
    let mut buttons = [false; 10];
    buttons[2] = true;
    let state = map_buttons(buttons, [false; 10], &settings);
    assert!(state.dive && !state.jump && !state.confirm);
    assert!(!map_buttons(buttons, buttons, &settings).dive);
    settings.bind(0, 2);
    assert_eq!(settings.bindings[1], 0);
    let state = map_buttons(buttons, [false; 10], &settings);
    assert!(state.jump && !state.dive && !state.confirm);
    buttons = [false; 10];
    buttons[0] = true;
    assert!(map_buttons(buttons, [false; 10], &settings).confirm);
}
#[test]
fn preferences_migrate_old_deadzones_and_reject_duplicate_bindings() {
    let old = Settings::decode("deadzone=15\n");
    assert_eq!(old.deadzone_percent, 15);
    assert_eq!(old.look_deadzone_percent, 15);
    let settings = Settings {
        look_deadzone_percent: 5,
        camera_sensitivity: 175,
        invert_y: true,
        auto_camera: true,
        bindings: [2, 0, 5, 3],
        vibration: false,
        volume: 30,
        ..Default::default()
    };
    assert_eq!(Settings::decode(&settings.encode()), settings);
    let bad = Settings::decode(
        "bindings=0,0,7,3\ncamera_sensitivity=999\nlook_deadzone=100\nvolume=1000",
    );
    assert_eq!(bad, Settings::default());
}
#[test]
fn vsync_defaults_on_and_round_trips() {
    assert!(Settings::default().vsync);
    let off = Settings {
        vsync: false,
        ..Default::default()
    };
    assert_eq!(Settings::decode(&off.encode()), off);
    assert!(Settings::decode("vsync=oops\n").vsync);
}
