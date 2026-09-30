//! Traverse the authored route with the real capsule and only gameplay input.
//! A small lookahead pilot waits for reachable windows; no position/velocity
//! writes occur after the initial spawn. This catches impossible course gaps.
use glam::{vec3, Vec2, Vec3};
use stride::{
    course::{Run, Surface},
    engine::{
        physics::{Input, Move, Player},
        FIXED_DT,
    },
    world::World,
};

fn input_toward(p: &Player, target: Vec3, sprint: bool, air: bool) -> Input {
    let error = vec3(target.x - p.pos.x, 0., target.z - p.pos.z);
    let desired = error * if air { 4. } else { 5. } - if air { p.air_carry } else { Vec3::ZERO };
    let speed = if sprint { 15. } else { 7.2 };
    Input {
        movement: Vec2::new(desired.x, -desired.z).clamp_length_max(speed) / speed,
        sprint,
        jump_held: true,
        ..Default::default()
    }
}
fn plan(w: &World, p: &Player, next: usize) -> Option<Vec<Input>> {
    for sprint in [false, true] {
        for prep in [0, 12, 24] {
            for eta in [0.45, 0.6, 0.75, 0.9, 1.05] {
                let node = &w.course.nodes[next];
                let target = if next == 32 {
                    vec3(228., 29., 112.)
                } else {
                    match node.surface {
                        Surface::Moving(i) => {
                            w.platforms[i].center_at(w.time + eta + prep as f64 * FIXED_DT as f64)
                                + Vec3::Y * w.platforms[i].size.y * 0.5
                        }
                        _ => node.position(w),
                    }
                };
                if (target.y - p.pos.y) > 3.5 || (target - p.pos).length() > 20. {
                    continue;
                }
                let mut world = w.clone();
                let mut player = p.clone();
                let mut commands = vec![];
                for _ in 0..prep {
                    let input = input_toward(&player, target, sprint, false);
                    world.advance(FIXED_DT);
                    player.step(input, 0., &world, FIXED_DT);
                    commands.push(input);
                }
                if !player.grounded {
                    continue;
                }
                for t in 0..180 {
                    let mut input = input_toward(&player, target, sprint, true);
                    input.jump = t == 0;
                    world.advance(FIXED_DT);
                    player.step(input, 0., &world, FIXED_DT);
                    commands.push(input);
                    if node.reached(&world, &player)
                        || (next == 32
                            && player.grounded
                            && player.pos.z > 110.25
                            && player.pos.z < 114.
                            && (player.pos.y - 29.).abs() < 0.01)
                    {
                        return Some(commands);
                    }
                    if player.crushed || player.pos.y < p.pos.y - 3. || (t > 2 && player.grounded) {
                        break;
                    }
                }
            }
        }
    }
    None
}

#[test]
fn full_skyway_route_is_traversable_at_different_platform_phases() {
    for phase in [0., 2.7] {
        let mut w = World::default();
        for _ in 0..(phase / FIXED_DT) as usize {
            w.advance(FIXED_DT);
        }
        let mut p = Player::default();
        p.respawn(w.course.nodes[0].position(&w) + Vec3::Y * 0.08);
        let mut run = Run::new(&w.course, None);
        let mut commands = std::collections::VecDeque::new();
        let mut last_progress = 0;
        for tick in 0..60000 {
            let mut input = Input::default();
            if let Some(command) = commands.pop_front() {
                input = command;
            } else if p.grounded {
                if run.next == 32 && p.pos.z >= 110. {
                    input = input_toward(&p, w.course.nodes[32].position(&w), false, false);
                    input.dive = p.motion == Move::Normal && p.pos.z > 112.5 && p.pos.z < 119.;
                    input.jump = p.motion == Move::Slide && p.pos.z > 120.2;
                } else if p.motion == Move::Normal {
                    if tick % 8 == 0 {
                        if let Some(plan) = plan(&w, &p, run.next) {
                            commands = plan.into();
                            input = commands.pop_front().unwrap();
                        }
                    }
                    if commands.is_empty() && !input.jump {
                        let current = &w.course.nodes[run.next - 1];
                        let slab = current.solid(&w);
                        let target = w.course.nodes[run.next].position(&w);
                        // Approach the departure edge with a safe 0.65 m inset.
                        let safe = vec3(
                            target.x.clamp(slab.min.x + 0.65, slab.max.x - 0.65),
                            p.pos.y,
                            target.z.clamp(slab.min.z + 0.65, slab.max.z - 0.65),
                        );
                        input = input_toward(&p, safe, run.next == 31, false);
                    }
                }
            }
            w.advance(FIXED_DT);
            p.step(input, 0., &w, FIXED_DT);
            let before = run.next;
            run.advance(&w, &p);
            if run.next != before {
                commands.clear();
                last_progress = tick;
                println!(
                    "phase {phase} reached {} at {:.1}s {:?}",
                    run.next, w.time, p.pos
                );
            }
            if run.finished {
                break;
            }
            assert!(
                p.pos.y >= w.course.nodes[run.checkpoint].position(&w).y - 6.,
                "fell at node {} phase {phase} pos {:?}",
                run.next,
                p.pos
            );
            assert!(
                tick - last_progress < 6000,
                "no reachable window at node {} phase {phase} pos {:?}",
                run.next,
                p.pos
            );
        }
        assert!(run.finished, "route incomplete at node {}", run.next);
        assert!(
            p.long_jumps > 0 && p.dives > 0,
            "route must exercise long jumps and diving"
        );
    }
}
