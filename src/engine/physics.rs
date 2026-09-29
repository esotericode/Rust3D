use crate::world::{World, SPAWN};
use glam::{vec3, Vec2, Vec3};

pub const RADIUS: f32 = 0.32;
pub const HEIGHT: f32 = 1.65;
const GRAVITY: f32 = 24.0;

#[derive(Clone, Copy, Default)]
pub struct Input {
    pub movement: Vec2,
    pub sprint: bool,
    pub jump: bool,
}

#[derive(Clone, Debug)]
pub struct Player {
    /// Centre of the feet; collision is an upright box.
    pub pos: Vec3,
    pub velocity: Vec3,
    pub grounded: bool,
    pub facing: f32,
    pub action: &'static str,
    pub wall_normal: Vec3,
    pub jumps: u32,
    pub wall_kicks: u32,
    pub long_jumps: u32,
    coyote: f32,
    jump_buffer: f32,
    wall_grace: f32,
    kick_lock: f32,
    last_wall: Vec3,
    long_air: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            pos: SPAWN,
            velocity: Vec3::ZERO,
            grounded: true,
            facing: 0.,
            action: "READY",
            wall_normal: Vec3::ZERO,
            jumps: 0,
            wall_kicks: 0,
            long_jumps: 0,
            coyote: 0.1,
            jump_buffer: 0.,
            wall_grace: 0.,
            kick_lock: 0.,
            last_wall: Vec3::ZERO,
            long_air: false,
        }
    }
}

impl Player {
    pub fn respawn(&mut self, pos: Vec3) {
        let counts = (self.jumps, self.wall_kicks, self.long_jumps);
        *self = Self::default();
        self.pos = pos;
        self.grounded = false;
        self.coyote = 0.;
        (self.jumps, self.wall_kicks, self.long_jumps) = counts;
    }

    pub fn speed(&self) -> f32 {
        Vec2::new(self.velocity.x, self.velocity.z).length()
    }

    pub fn step(&mut self, input: Input, yaw: f32, world: &World, dt: f32) {
        self.coyote = if self.grounded {
            0.10
        } else {
            (self.coyote - dt).max(0.)
        };
        self.jump_buffer = if input.jump {
            0.12
        } else {
            (self.jump_buffer - dt).max(0.)
        };
        self.wall_grace = (self.wall_grace - dt).max(0.);
        self.kick_lock = (self.kick_lock - dt).max(0.);
        let forward = vec3(-yaw.sin(), 0., -yaw.cos());
        let right = vec3(yaw.cos(), 0., -yaw.sin());
        let wish = (right * input.movement.x + forward * input.movement.y).normalize_or_zero();
        if wish.length_squared() > 0. {
            self.facing = (-wish.x).atan2(-wish.z);
        }
        let max_speed = if self.long_air {
            13.5
        } else if input.sprint {
            9.0
        } else {
            6.2
        };
        let accel = if self.grounded { 45. } else { 13. };
        if self.kick_lock <= 0. {
            let target = wish * max_speed;
            let horizontal = vec3(self.velocity.x, 0., self.velocity.z);
            let delta = if wish == Vec3::ZERO && !self.grounded {
                Vec3::ZERO
            } else {
                target - horizontal
            };
            let change = delta.clamp_length_max(accel * dt);
            self.velocity.x += change.x;
            self.velocity.z += change.z;
        }
        if self.jump_buffer > 0. && self.coyote > 0. {
            self.velocity.y = if input.sprint && self.speed() > 3. {
                7.8
            } else {
                9.8
            };
            self.long_air = input.sprint && self.speed() > 3.;
            if self.long_air {
                let launch = if wish.length_squared() > 0. {
                    wish
                } else {
                    vec3(self.velocity.x, 0., self.velocity.z).normalize_or_zero()
                };
                self.velocity.x = launch.x * 13.5;
                self.velocity.z = launch.z * 13.5;
                self.long_jumps += 1;
                self.action = "LONG JUMP";
            } else {
                self.action = "JUMP";
            }
            self.jumps += 1;
            self.grounded = false;
            self.coyote = 0.;
            self.jump_buffer = 0.;
        } else if self.jump_buffer > 0.
            && !self.grounded
            && self.wall_grace > 0.
            && self.kick_lock <= 0.
            && self.wall_normal.dot(self.last_wall) < 0.8
        {
            self.velocity = self.wall_normal * 9.8 + Vec3::Y * 10.8 + wish * 1.2;
            self.last_wall = self.wall_normal;
            self.kick_lock = 0.18;
            self.jump_buffer = 0.;
            self.wall_grace = 0.;
            self.long_air = false;
            self.wall_kicks += 1;
            self.action = "WALL KICK";
        }
        let old_y = self.pos.y;
        let was_grounded = self.grounded;
        self.move_axis(world, 0, dt);
        self.move_axis(world, 2, dt);
        // Climb the continuous ramp surface, but never snap an airborne player up.
        if was_grounded && self.velocity.y <= 0. {
            for r in &world.ramps {
                if let Some(h) = r.height(self.pos.x, self.pos.z) {
                    if h >= old_y - 0.15 && h <= old_y + 0.25 && self.pos.y <= old_y + 0.001 {
                        self.pos.y = h;
                    }
                }
            }
        }
        let before_y = self.pos.y;
        self.velocity.y = (self.velocity.y - GRAVITY * dt).max(-35.);
        self.pos.y += self.velocity.y * dt;
        self.grounded = false;
        for s in &world.solids {
            if !overlaps(self.pos, s.min, s.max) {
                continue;
            }
            if self.velocity.y <= 0. && before_y >= s.max.y - 0.02 && self.pos.y <= s.max.y {
                self.pos.y = s.max.y;
                self.land();
            } else if self.velocity.y > 0.
                && before_y + HEIGHT <= s.min.y + 0.01
                && self.pos.y + HEIGHT >= s.min.y
            {
                self.pos.y = s.min.y - HEIGHT;
                self.velocity.y = 0.;
            }
        }
        for r in &world.ramps {
            if let Some(h) = r.height(self.pos.x, self.pos.z) {
                if self.velocity.y <= 0. && before_y >= h - 0.15 && self.pos.y <= h {
                    self.pos.y = h;
                    self.land();
                }
            }
        }
        if !self.grounded && self.wall_grace > 0. && self.velocity.y < -3.0 {
            self.velocity.y = -3.0;
            self.action = "WALL SLIDE";
        }
        if self.grounded {
            self.action = if self.speed() > 0.5 {
                if input.sprint {
                    "SPRINT"
                } else {
                    "RUN"
                }
            } else {
                "READY"
            };
        } else if self.velocity.y < 0. && self.wall_grace <= 0. {
            self.action = "AIRBORNE";
        }
    }

    fn land(&mut self) {
        self.grounded = true;
        self.velocity.y = 0.;
        self.long_air = false;
        self.last_wall = Vec3::ZERO;
        self.wall_grace = 0.;
    }

    fn contact(&mut self, axis: usize, direction: f32) {
        let mut normal = Vec3::ZERO;
        normal[axis] = -direction;
        self.wall_normal = normal;
        self.wall_grace = 0.10;
        self.velocity[axis] = 0.;
    }

    fn move_axis(&mut self, world: &World, axis: usize, dt: f32) {
        let delta = self.velocity[axis] * dt;
        if delta.abs() < 0.000001 {
            return;
        }
        self.pos[axis] += delta;
        for s in &world.solids {
            if self.pos.y >= s.max.y - 0.01
                || self.pos.y + HEIGHT <= s.min.y + 0.01
                || !overlaps(self.pos, s.min, s.max)
            {
                continue;
            }
            // Small surface rises bridge a wedge to its landing without catching
            // the front of the character box on the last few centimetres.
            if self.grounded && s.max.y - self.pos.y <= 0.25 {
                let raised = vec3(self.pos.x, s.max.y, self.pos.z);
                let clear = !world.solids.iter().any(|other| {
                    overlaps(raised, other.min, other.max)
                        && raised.y < other.max.y - 0.01
                        && raised.y + HEIGHT > other.min.y + 0.01
                });
                if clear {
                    self.pos.y = s.max.y;
                    continue;
                }
            }
            self.pos[axis] = if delta > 0. {
                s.min[axis] - RADIUS
            } else {
                s.max[axis] + RADIUS
            };
            self.contact(axis, delta.signum());
        }
        for r in &world.ramps {
            let sample = vec3(
                self.pos.x.clamp(r.min.x, r.max.x),
                0.,
                self.pos.z.clamp(r.min.z, r.max.z),
            );
            if overlaps(self.pos, r.min, r.max) && self.pos.y + HEIGHT > r.min.y {
                if let Some(h) = r.height(sample.x, sample.z) {
                    if self.pos.y + 0.25 < h {
                        self.pos[axis] = if delta > 0. {
                            r.min[axis] - RADIUS
                        } else {
                            r.max[axis] + RADIUS
                        };
                        self.contact(axis, delta.signum());
                    }
                }
            }
        }
    }
}

fn overlaps(p: Vec3, min: Vec3, max: Vec3) -> bool {
    p.x + RADIUS > min.x + 0.001
        && p.x - RADIUS < max.x - 0.001
        && p.z + RADIUS > min.z + 0.001
        && p.z - RADIUS < max.z - 0.001
}
