use glam::{vec2, vec3, Vec2, Vec3};
use stride::{
    engine::{
        camera::Camera,
        physics::{momentum_gain, Input, Move, Player},
        terrain::{Terrain, MAP_SIZE, REGIONS},
        FIXED_DT,
    },
    world::{Solid, World, NAVY},
};

fn empty() -> World {
    World {
        terrain: None,
        solids: vec![],
        ramps: vec![],
        beacons: vec![],
        platforms: vec![],
        course: Default::default(),
        time: 0.,
    }
}
fn flat() -> World {
    let mut w = empty();
    w.solids.push(Solid::new(
        vec3(0., -0.5, 0.),
        vec3(20000., 1., 20000.),
        NAVY,
    ));
    w
}
fn slope(grade: f32) -> World {
    let mut w = empty();
    w.terrain = Some(Terrain::from_fn(vec2(-300., -300.), 60, 60, 10., |x, _| {
        100. + grade * x
    }));
    w
}
fn at(pos: Vec3) -> Player {
    let mut p = Player::default();
    p.respawn(pos);
    p.grounded = true;
    p
}
fn forward() -> Input {
    Input {
        movement: Vec2::Y,
        sprint: true,
        crouch: true,
        jump_held: true,
        ..Default::default()
    }
}

#[test]
fn landscape_is_sixteen_times_larger_seeded_and_has_clear_region_spawns() {
    assert_eq!(MAP_SIZE.x * MAP_SIZE.y / (360. * 420.), 16.);
    let a = World::default();
    let b = World::default();
    assert_eq!(
        a.terrain.as_ref().unwrap().heights,
        b.terrain.as_ref().unwrap().heights
    );
    assert_eq!(
        a.solids.iter().map(|s| s.min).collect::<Vec<_>>(),
        b.solids.iter().map(|s| s.min).collect::<Vec<_>>()
    );
    assert!(
        a.terrain
            .as_ref()
            .unwrap()
            .heights
            .iter()
            .copied()
            .fold(0., f32::max)
            > 200.
    );
    assert!(a.solids.len() > 280 && a.ramps.len() == 51);
    for r in &REGIONS {
        let (h, _) = a
            .terrain
            .as_ref()
            .unwrap()
            .sample(r.point.x, r.point.y)
            .unwrap();
        let p = vec3(r.point.x, h + 0.05, r.point.y);
        assert!(
            !a.collision_solids().any(|s| s.contains(p, 0.5)),
            "{} spawn blocked",
            r.name
        );
    }
}
#[test]
fn highland_ramps_sit_on_the_terrain_without_open_undersides() {
    let w = World::default();
    let t = w.terrain.as_ref().unwrap();
    for (i, r) in w.ramps.iter().enumerate() {
        assert!(r.base <= r.min.y, "ramp {i} base above its low edge");
        let mut x = r.min.x;
        while x <= r.max.x {
            let mut z = r.min.z;
            while z <= r.max.z {
                // Support is the terrain or a block top (the lab ramps stand
                // on the courtyard slab, above the terrain beneath it).
                let ground = w
                    .solids
                    .iter()
                    .filter(|s| s.contains(vec3(x, s.max.y, z), 0.) && s.max.y <= r.base + 0.01)
                    .map(|s| s.max.y)
                    .chain(t.sample(x, z).map(|s| s.0))
                    .fold(f32::NEG_INFINITY, f32::max);
                assert!(
                    ground >= r.base - 0.01,
                    "ramp {i} floats {:.2} m above the ground at {x:.1}, {z:.1}",
                    r.base - ground
                );
                z += 1.;
            }
            x += 1.;
        }
    }
}
#[test]
fn terrain_does_not_pull_grounded_players_through_the_original_lab_or_blocks() {
    let w = World::default();
    let mut p = Player::default();
    for _ in 0..120 {
        p.step(Input::default(), 0., &w, FIXED_DT);
        assert!(
            p.grounded && !p.crushed && p.pos.y.abs() < 0.001,
            "lab support lost: {p:?}"
        );
    }
    let mut w = slope(0.4);
    w.solids
        .push(Solid::new(vec3(0., 99.8, 0.), vec3(12., 2.4, 12.), NAVY));
    let mut p = at(vec3(0., 101., 0.));
    for _ in 0..120 {
        p.step(Input::default(), 0., &w, FIXED_DT);
        assert!(
            p.grounded && !p.crushed && (p.pos.y - 101.).abs() < 0.001,
            "block support lost: {p:?}"
        );
    }
}

#[test]
fn heightfield_collision_matches_rendered_triangle_centres_and_has_no_tile_seams() {
    let t = Terrain::highlands();
    for m in t.meshes() {
        assert!(m.vertices.len() < 60000);
        for triangle in m.indices.as_chunks::<3>().0.iter().step_by(37) {
            let p = triangle
                .iter()
                .map(|&i| Vec3::from_array(m.vertices[i as usize].pos))
                .sum::<Vec3>()
                / 3.;
            let (h, n) = t.sample(p.x, p.z).unwrap();
            assert!(
                (h - p.y).abs() < 0.003,
                "triangle/collision mismatch at {p:?}"
            );
            assert!(n.y > 0. && (n.length() - 1.).abs() < 0.0001);
        }
    }
    assert!(t.sample(-621., 0.).is_none());
}
#[test]
fn downhill_adds_energy_uphill_spends_it_and_feet_follow_the_grade() {
    let mut outcomes = vec![];
    for grade in [-0.4, 0.4] {
        let w = slope(grade);
        let mut p = at(vec3(0., 100., 0.));
        p.velocity = vec3(20., grade * 20., 0.);
        for _ in 0..120 {
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
            let h = w
                .terrain
                .as_ref()
                .unwrap()
                .sample(p.pos.x, p.pos.z)
                .unwrap()
                .0;
            assert!(
                p.grounded && (p.pos.y - h).abs() < 0.03,
                "lost slope contact {p:?}"
            );
        }
        outcomes.push(p.speed());
    }
    assert!(
        outcomes[0] > 23. && outcomes[1] < 17.,
        "up/down speeds {outcomes:?}"
    );
}
#[test]
fn steep_faces_slide_even_with_uphill_input_and_can_be_jumped_off() {
    let w = slope(2.);
    let mut p = at(vec3(0., 100., 0.));
    for _ in 0..240 {
        p.step(
            Input {
                movement: Vec2::X,
                ..Default::default()
            },
            0.,
            &w,
            FIXED_DT,
        );
    }
    assert!(
        p.pos.x < -3. && p.grounded && p.ground_normal.y < 0.64,
        "{p:?}"
    );
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
    assert!(!p.grounded && p.velocity.y > 8.);
}
#[test]
fn uphill_jumps_dives_and_rollouts_keep_the_slopes_upward_momentum() {
    let w = slope(0.6);
    for motion in [Move::Normal, Move::Slide] {
        for (jump, crouch, dive) in [
            (true, false, false),
            (true, true, false),
            (false, false, true),
        ] {
            let mut p = at(vec3(0., 100., 0.));
            p.velocity = vec3(60., 36., 0.);
            p.motion = motion;
            p.step(
                Input {
                    movement: Vec2::X,
                    sprint: true,
                    crouch,
                    jump,
                    jump_held: true,
                    dive,
                },
                0.,
                &w,
                FIXED_DT,
            );
            assert!(
                !p.grounded && p.velocity.y > 40.,
                "uphill launch lost momentum: {p:?}"
            );
            for _ in 0..12 {
                p.step(
                    Input {
                        movement: Vec2::X,
                        sprint: true,
                        jump_held: true,
                        ..Default::default()
                    },
                    0.,
                    &w,
                    FIXED_DT,
                );
                let h = w
                    .terrain
                    .as_ref()
                    .unwrap()
                    .sample(p.pos.x, p.pos.z)
                    .unwrap()
                    .0;
                assert!(
                    p.pos.y > h && !p.grounded,
                    "uphill launch snapped to surface: {p:?}"
                );
            }
        }
    }
}

#[test]
fn fast_runners_fly_off_crests_and_slower_runners_follow_the_ground() {
    let mut w = empty();
    // A 6 m rise, 25 m wide: about 0.019 /m of curvature at the crest.
    w.terrain = Some(Terrain::from_fn(vec2(-200., -40.), 80, 16, 5., |x, _| {
        6. * (-(x / 25.).powi(2)).exp()
    }));
    let airtime = |speed: f32| {
        let mut p = at(vec3(-150., 0., 0.));
        p.velocity = vec3(speed, 0., 0.);
        let mut airborne = 0;
        for _ in 0..(400. / speed / FIXED_DT) as usize {
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
            let ground = w
                .terrain
                .as_ref()
                .unwrap()
                .sample(p.pos.x, p.pos.z)
                .unwrap()
                .0;
            assert!(p.pos.y >= ground - 0.01, "below the ground: {p:?}");
            airborne += !p.grounded as usize;
            if p.pos.x > 100. {
                break;
            }
        }
        airborne as f32 * FIXED_DT
    };
    assert_eq!(airtime(25.), 0., "a moderate run should follow the hill");
    let flight = airtime(50.);
    assert!(
        flight > 0.15,
        "a fast run should leave the crest: {flight} s"
    );
}
#[test]
fn hopping_cannot_climb_a_face_too_steep_to_walk() {
    let w = slope(1.5);
    let mut p = at(vec3(0., 100., 0.));
    for i in 0..360 {
        p.step(
            Input {
                movement: Vec2::X,
                jump: i % 4 == 0,
                jump_held: true,
                ..Default::default()
            },
            0.,
            &w,
            FIXED_DT,
        );
        // A hop may rise briefly, but every landing must be lower down.
        assert!(!p.grounded || p.pos.y <= 100.001, "landed higher: {p:?}");
    }
    assert!(p.pos.y < 100., "{p:?}");
}
#[test]
fn touchdowns_along_a_slope_are_soft_and_drops_onto_it_are_hard() {
    let w = slope(-0.5);
    let mut along = at(vec3(0., 100., 0.));
    along.grounded = false;
    along.velocity = vec3(20., -10., 0.);
    along.step(
        Input {
            movement: Vec2::X,
            sprint: true,
            ..Default::default()
        },
        0.,
        &w,
        FIXED_DT,
    );
    assert!(along.grounded && along.landings == 0, "{along:?}");
    let mut drop = at(vec3(0., 101., 0.));
    drop.grounded = false;
    drop.velocity = vec3(0., -10., 0.);
    for _ in 0..30 {
        drop.step(Input::default(), 0., &w, FIXED_DT);
        if drop.grounded {
            break;
        }
    }
    assert!(
        drop.grounded && drop.landings == 1 && drop.impact_speed > 7.,
        "{drop:?}"
    );
}
#[test]
fn chained_long_jumps_build_speed_above_old_caps_with_diminishing_gains() {
    let w = flat();
    let mut p = at(Vec3::ZERO);
    let mut gains = vec![];
    for _ in 0..3600 {
        let jump = p.grounded;
        p.step(Input { jump, ..forward() }, 0., &w, FIXED_DT);
        if jump && p.long_jumps > 0 {
            gains.push(p.last_gain);
        }
    }
    assert!(p.long_jumps > 25 && p.peak_speed > 55., "{p:?}");
    assert!(gains.last().unwrap() < &gains[0]);
    assert!(momentum_gain(1000., 6.) > 0. && momentum_gain(1000., 6.) < momentum_gain(100., 6.));
}
#[test]
fn dive_rollout_cycles_build_speed_and_fast_dives_cannot_instantly_reverse() {
    let w = flat();
    let mut p = at(Vec3::ZERO);
    for _ in 0..2400 {
        p.step(
            Input {
                dive: p.grounded && p.motion == Move::Normal,
                jump: p.motion == Move::Slide,
                ..forward()
            },
            0.,
            &w,
            FIXED_DT,
        );
    }
    assert!(
        p.dives > 12 && p.rollouts > 12 && p.peak_speed > 50.,
        "{p:?}"
    );
    let mut fast = at(vec3(0., 8., 0.));
    fast.grounded = false;
    fast.velocity = vec3(80., 0., 0.);
    fast.step(
        Input {
            dive: true,
            movement: Vec2::NEG_X,
            ..Default::default()
        },
        0.,
        &w,
        FIXED_DT,
    );
    assert!(fast.velocity.x > 75. && fast.speed() > 80.);
}
#[test]
fn high_speed_braking_and_thin_wall_collisions_remain_reliable() {
    let mut w = flat();
    w.solids
        .push(Solid::new(vec3(2., 4., 0.), vec3(0.08, 8., 10.), NAVY));
    let mut p = at(Vec3::ZERO);
    p.velocity = vec3(300., 0., 0.);
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
    assert!(
        p.pos.x < 1.65 && p.velocity.x.abs() < 0.01,
        "tunnelled {p:?}"
    );
    let w = flat();
    let mut p = at(Vec3::ZERO);
    p.velocity = vec3(0., 0., -65.);
    for _ in 0..90 {
        p.step(
            Input {
                movement: Vec2::NEG_Y,
                ..Default::default()
            },
            0.,
            &w,
            FIXED_DT,
        );
    }
    assert!(p.velocity.z > 5. && p.speed() < 8.);
}
#[test]
fn air_coasts_and_reduced_analog_input_can_slow_down_for_precision_landings() {
    let w = flat();
    let mut p = at(vec3(0., 15., 0.));
    p.grounded = false;
    p.velocity = vec3(0., 0., -45.);
    for _ in 0..30 {
        p.step(Input::default(), 0., &w, FIXED_DT);
    }
    assert!((p.speed() - 45.).abs() < 0.01);
    for _ in 0..90 {
        p.step(
            Input {
                movement: Vec2::Y * 0.2,
                ..Default::default()
            },
            0.,
            &w,
            FIXED_DT,
        );
    }
    assert!(p.speed() < 20.);
}
#[test]
fn camera_stays_above_hills_and_speed_changes_do_not_change_physics() {
    let w = slope(-0.3);
    let mut p = at(vec3(0., 100., 0.));
    p.velocity = vec3(0., 0., -60.);
    let mut camera = Camera::new(p.pos);
    for _ in 0..120 {
        camera.follow(&p, false, &w, FIXED_DT);
    }
    assert!(camera.eye.is_finite());
    let h = w
        .terrain
        .as_ref()
        .unwrap()
        .sample(camera.eye.x, camera.eye.z)
        .unwrap()
        .0;
    assert!(camera.eye.y > h && p.speed() == 60.);
}
