use glam::{vec3, Vec2, Vec3};
use stride::{
    engine::{
        physics::{Input, Player, HEIGHT, RADIUS},
        FIXED_DT,
    },
    world::{Ramp, Solid, World, NAVY},
};

fn floor() -> World {
    World {
        solids: vec![Solid::new(vec3(0., -0.5, 0.), vec3(100., 1., 100.), NAVY)],
        ramps: vec![],
        beacons: vec![],
    }
}
fn player(pos: Vec3) -> Player {
    let mut p = Player::default();
    p.respawn(pos);
    p.grounded = true;
    p
}
fn steps(p: &mut Player, w: &World, input: Input, count: usize) {
    for _ in 0..count {
        p.step(input, 0., w, FIXED_DT);
    }
}
fn movement(x: f32, y: f32, sprint: bool) -> Input {
    Input {
        movement: Vec2::new(x, y),
        sprint,
        jump: false,
    }
}

#[test]
fn running_accelerates_and_stops_without_drifting() {
    let w = floor();
    let mut p = player(Vec3::ZERO);
    steps(&mut p, &w, movement(0., 1., false), 120);
    assert!((p.speed() - 6.2).abs() < 0.01 && p.pos.z < -5. && p.grounded);
    steps(&mut p, &w, Input::default(), 60);
    assert!(p.speed() < 0.01 && p.pos.y.abs() < 0.001);
}
#[test]
fn diagonal_input_is_normalized_and_camera_relative() {
    let w = floor();
    let mut a = player(Vec3::ZERO);
    let mut b = a.clone();
    steps(&mut a, &w, movement(1., 1., false), 120);
    steps(&mut b, &w, movement(1., 0., false), 120);
    assert!((a.speed() - b.speed()).abs() < 0.001);
    let mut c = player(Vec3::ZERO);
    for _ in 0..120 {
        c.step(
            movement(0., 1., false),
            std::f32::consts::FRAC_PI_2,
            &w,
            FIXED_DT,
        );
    }
    assert!(c.pos.x < -5. && c.pos.z.abs() < 0.001);
}
#[test]
fn jump_has_an_apex_and_lands_on_floor() {
    let w = floor();
    let mut p = player(Vec3::ZERO);
    let mut apex = 0_f32;
    p.step(
        Input {
            jump: true,
            ..Default::default()
        },
        0.,
        &w,
        FIXED_DT,
    );
    for _ in 0..150 {
        p.step(Input::default(), 0., &w, FIXED_DT);
        apex = apex.max(p.pos.y);
    }
    assert!(apex > 1.9 && apex < 2.1 && p.grounded && p.pos.y.abs() < 0.001);
    assert_eq!(p.jumps, 1);
}
#[test]
fn long_jump_clears_the_playground_gap() {
    let w = World::default();
    let mut p = player(vec3(0., 3., -16.));
    steps(&mut p, &w, movement(1., 0., true), 22);
    p.step(
        Input {
            jump: true,
            ..movement(1., 0., true)
        },
        0.,
        &w,
        FIXED_DT,
    );
    let mut landed = false;
    for _ in 0..90 {
        p.step(movement(1., 0., true), 0., &w, FIXED_DT);
        if p.grounded && p.pos.x > 8.5 {
            landed = true;
            break;
        }
    }
    assert!(landed, "landing position: {:?}", p.pos);
    assert_eq!(p.long_jumps, 1);
    assert!((p.pos.y - 3.).abs() < 0.001);
}
#[test]
fn ramp_can_be_walked_up_and_down() {
    let mut w = floor();
    w.ramps.push(Ramp {
        min: vec3(-3., 0., -8.),
        max: vec3(3., 2., 0.),
    });
    let mut p = player(vec3(0., 0., 1.));
    steps(&mut p, &w, movement(0., 1., false), 145);
    assert!(p.pos.y > 1.4 && p.grounded, "{:?}", p.pos);
    steps(&mut p, &w, movement(0., -1., false), 170);
    assert!(p.pos.y.abs() < 0.001 && p.grounded, "{:?}", p.pos);
}
#[test]
fn playground_ramps_join_their_landing_platforms() {
    let w = World::default();
    for (start, top) in [(vec3(-8., 0., 10.), 2.), (vec3(0., 0., -2.5), 3.)] {
        let mut p = player(start);
        steps(&mut p, &w, movement(0., 1., false), 220);
        assert!(
            p.pos.y >= top - 0.01 && p.grounded,
            "ramp landing: {:?}",
            p.pos
        );
    }
}
#[test]
fn block_sides_and_tunnel_ceiling_are_solid() {
    let mut w = floor();
    w.solids
        .push(Solid::new(vec3(2., 1., 0.), vec3(1., 2., 4.), NAVY));
    let mut p = player(Vec3::ZERO);
    steps(&mut p, &w, movement(1., 0., true), 120);
    assert!(p.pos.x <= 1.5 - RADIUS + 0.001);
    w.solids
        .push(Solid::new(vec3(0., 2.5, 0.), vec3(2., 0.5, 2.), NAVY));
    p = player(Vec3::ZERO);
    p.step(
        Input {
            jump: true,
            ..Default::default()
        },
        0.,
        &w,
        FIXED_DT,
    );
    for _ in 0..25 {
        p.step(Input::default(), 0., &w, FIXED_DT);
        assert!(p.pos.y + HEIGHT <= 2.251);
    }
}
#[test]
fn wall_slide_and_alternating_kicks_gain_height() {
    let mut w = floor();
    w.solids
        .push(Solid::new(vec3(-2., 4., 0.), vec3(1., 8., 10.), NAVY));
    w.solids
        .push(Solid::new(vec3(2., 4., 0.), vec3(1., 8., 10.), NAVY));
    let mut p = player(vec3(1.17, 2., 0.));
    p.grounded = false;
    p.velocity = vec3(4., -10., 0.);
    p.step(movement(1., 0., false), 0., &w, FIXED_DT);
    assert_eq!(p.wall_normal, Vec3::NEG_X);
    assert!(p.velocity.y >= -3.01);
    p.step(
        Input {
            jump: true,
            ..Default::default()
        },
        0.,
        &w,
        FIXED_DT,
    );
    assert!(p.velocity.x < -8. && p.velocity.y > 10. && p.wall_kicks == 1);
    let first_height = p.pos.y;
    for _ in 0..70 {
        p.step(movement(-1., 0., true), 0., &w, FIXED_DT);
        if p.wall_normal == Vec3::X {
            break;
        }
    }
    p.step(
        Input {
            jump: true,
            ..Default::default()
        },
        0.,
        &w,
        FIXED_DT,
    );
    assert!(
        p.velocity.x > 8. && p.pos.y > first_height && p.wall_kicks == 2,
        "{:?}",
        p
    );
}
#[test]
fn jump_buffer_triggers_after_landing() {
    let w = floor();
    let mut p = player(vec3(0., 0.1, 0.));
    p.grounded = false;
    p.velocity.y = -3.;
    p.step(
        Input {
            jump: true,
            ..Default::default()
        },
        0.,
        &w,
        FIXED_DT,
    );
    steps(&mut p, &w, Input::default(), 6);
    assert!(p.velocity.y > 7. && p.jumps == 1);
}
#[test]
fn walking_off_edge_keeps_a_short_coyote_jump() {
    let w = World {
        solids: vec![Solid::new(vec3(0., -0.5, 0.), vec3(2., 1., 2.), NAVY)],
        ramps: vec![],
        beacons: vec![],
    };
    let mut p = player(vec3(1.31, 0., 0.));
    p.velocity.x = 6.2;
    steps(&mut p, &w, movement(1., 0., false), 3);
    p.step(
        Input {
            jump: true,
            ..movement(1., 0., false)
        },
        0.,
        &w,
        FIXED_DT,
    );
    assert!(p.velocity.y > 9. && p.jumps == 1);
}
