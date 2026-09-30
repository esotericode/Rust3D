use glam::{vec3, Vec2, Vec3};
use stride::{
    course::{MovingPlatform, Run},
    engine::{
        physics::{Input, Player},
        FIXED_DT,
    },
    world::{Solid, World, NAVY},
};

fn rig(travel: Vec3) -> (World, Player) {
    let platform = MovingPlatform::new(vec3(0., 5., 0.), vec3(6., 0.7, 6.), travel, 8., 0.);
    let mut p = Player::default();
    p.respawn(vec3(0., 5.08, 0.));
    let mut w = World {
        terrain: None,
        solids: vec![],
        ramps: vec![],
        beacons: vec![],
        platforms: vec![platform],
        course: Default::default(),
        time: 0.,
    };
    for _ in 0..20 {
        tick(&mut w, &mut p, Input::default());
    }
    assert_eq!(p.ground_platform, Some(0));
    (w, p)
}
fn tick(w: &mut World, p: &mut Player, i: Input) {
    w.advance(FIXED_DT);
    p.step(i, 0., w, FIXED_DT);
}
#[test]
fn riders_remain_stable_through_horizontal_vertical_and_diagonal_reversals() {
    for travel in [vec3(12., 0., 0.), vec3(0., 8., 0.), vec3(9., 5., -8.)] {
        let (mut w, mut p) = rig(travel);
        for _ in 0..1920 {
            tick(&mut w, &mut p, Input::default());
            let center = w.platforms[0].solid.max - vec3(3., 0., 3.);
            assert!(
                (p.pos - center).length() < 0.002,
                "lost rider at {}: {:?} vs {:?}",
                w.time,
                p.pos,
                center
            );
            assert!(p.grounded && !p.crushed);
        }
    }
}
#[test]
fn jumps_inherit_shuttle_motion_and_landings_release_it() {
    let (mut w, mut p) = rig(vec3(12., 0., 0.));
    for _ in 0..180 {
        tick(&mut w, &mut p, Input::default());
    }
    let speed = w.platforms[0].velocity;
    let before = p.pos;
    tick(
        &mut w,
        &mut p,
        Input {
            jump: true,
            jump_held: true,
            ..Default::default()
        },
    );
    assert!(p.air_carry.x > 4. && (p.air_carry.x - speed.x).abs() < 0.1);
    assert!(p.ground_platform.is_none());
    for _ in 0..15 {
        tick(
            &mut w,
            &mut p,
            Input {
                jump_held: true,
                ..Default::default()
            },
        );
    }
    assert!(p.pos.x - before.x > 0.6);
    w.solids
        .push(Solid::new(vec3(20., -0.5, 0.), vec3(100., 1., 100.), NAVY));
    for _ in 0..180 {
        tick(&mut w, &mut p, Input::default());
    }
    assert!(p.grounded && p.air_carry == Vec3::ZERO);
}
#[test]
fn lift_jump_inherits_vertical_speed_and_walking_off_preserves_coyote() {
    let (mut w, mut p) = rig(vec3(0., 8., 0.));
    for _ in 0..180 {
        tick(&mut w, &mut p, Input::default());
    }
    tick(
        &mut w,
        &mut p,
        Input {
            jump: true,
            jump_held: true,
            ..Default::default()
        },
    );
    assert!((p.velocity.y - (11.2 + w.platforms[0].velocity.y - 28. * FIXED_DT)).abs() < 0.001);
    let (mut w, mut p) = rig(vec3(10., 0., 0.));
    for _ in 0..180 {
        tick(&mut w, &mut p, Input::default());
    }
    while p.grounded {
        tick(
            &mut w,
            &mut p,
            Input {
                movement: Vec2::Y,
                ..Default::default()
            },
        );
    }
    assert!(p.air_carry.x > 1.);
    tick(
        &mut w,
        &mut p,
        Input {
            jump: true,
            jump_held: true,
            ..Default::default()
        },
    );
    assert!(p.velocity.y > 10. && p.air_carry.x > 1.);
}
#[test]
fn rising_lift_catches_falling_capsule_and_head_pinch_is_detected() {
    let (mut w, mut p) = rig(vec3(0., 8., 0.));
    p.respawn(vec3(0., 9., 0.));
    for _ in 0..200 {
        tick(&mut w, &mut p, Input::default());
    }
    assert_eq!(p.ground_platform, Some(0));
    w.solids
        .push(Solid::new(vec3(0., 12., 0.), vec3(10., 0.5, 10.), NAVY));
    let mut pinched = false;
    for _ in 0..400 {
        tick(&mut w, &mut p, Input::default());
        if p.crushed {
            pinched = true;
            break;
        }
    }
    assert!(pinched);
}
#[test]
fn platform_side_pushes_and_static_walls_do_not_allow_penetration() {
    let (mut w, mut p) = rig(vec3(12., 0., 0.));
    w.solids
        .push(Solid::new(vec3(7., 7., 0.), vec3(1., 8., 12.), NAVY));
    for _ in 0..600 {
        tick(&mut w, &mut p, Input::default());
        assert!(p.pos.x <= 6.181);
    }
    assert!(p.ground_platform.is_none());
}
#[test]
fn phase_reset_and_render_interpolation_do_not_move_physics() {
    let (mut w, mut p) = rig(vec3(10., 0., 0.));
    for _ in 0..180 {
        tick(&mut w, &mut p, Input::default());
    }
    let before = w.platforms[0].solid.clone();
    let half = w.platforms[0].rendered_center(0.5);
    assert!((half - w.platforms[0].previous_center).length() < w.platforms[0].delta.length());
    assert_eq!(before.min, w.platforms[0].solid.min);
    w.reset_platforms();
    assert_eq!(w.time, 0.);
    assert_eq!(w.platforms[0].delta, Vec3::ZERO);
    assert_eq!(w.platforms[0].solid.max, vec3(3., 5., 3.));
}
#[test]
fn course_checkpoints_are_fixed_ordered_and_practice_stops_at_next_deck() {
    let w = World::default();
    assert_eq!(w.course.nodes.len(), 46);
    assert_eq!(w.platforms.len(), 16);
    assert_eq!(w.course.checkpoints.len(), 9);
    for s in 0..8 {
        let mut r = Run::new(&w.course, Some(s));
        let last = w.course.checkpoints[s + 1];
        let checkpoint = r.checkpoint;
        r.clock(0.5, false);
        assert_eq!(r.elapsed, 0.);
        r.clock(0.5, true);
        assert_eq!(r.elapsed, 0.5);
        let mut p = Player::default();
        p.respawn(w.course.nodes[last].position(&w));
        p.grounded = true;
        assert!(!r.advance(&w, &p));
        assert_eq!(r.checkpoint, checkpoint);
        r.next = last;
        assert!(r.advance(&w, &p));
        assert!(r.finished);
        r.clock(2., true);
        assert_eq!(r.elapsed, 0.5);
    }
}
#[test]
fn retry_replays_from_last_fixed_checkpoint_and_preserves_clock() {
    let w = World::default();
    let mut r = Run::new(&w.course, None);
    r.next = 10;
    let mut p = Player::default();
    p.respawn(w.course.nodes[10].position(&w));
    p.grounded = true;
    assert!(r.advance(&w, &p));
    r.clock(10., true);
    r.next = 15;
    r.retry();
    assert_eq!((r.checkpoint, r.next, r.failures), (10, 11, 1));
    assert_eq!(r.elapsed, 10.);
}
