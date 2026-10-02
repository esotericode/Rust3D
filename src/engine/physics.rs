use crate::{
    engine::camera::angle_delta,
    world::{World, SPAWN},
};
use glam::{vec3, Vec2, Vec3};

pub const RADIUS: f32 = 0.32;
pub const HEIGHT: f32 = 1.65;
const GRAVITY: f32 = 28.0;
pub const WALK_SPEED: f32 = 7.2;
pub const SPRINT_SPEED: f32 = 10.5;
pub const WALL_GRACE: f32 = 0.035;
pub const WALKABLE_NORMAL_Y: f32 = 0.64;

/// Earned speed has no hard cap. Each manoeuvre supplies less extra speed as
/// kinetic energy rises; drag, braking and collisions provide practical limits.
pub fn momentum_gain(speed: f32, impulse: f32) -> f32 {
    impulse / (1. + (speed.max(0.) / 22.).powf(1.5))
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Move {
    #[default]
    Normal,
    Dive,
    Slide,
    Rollout,
}

#[derive(Clone, Copy, Default)]
pub struct Input {
    pub movement: Vec2,
    pub sprint: bool,
    pub jump: bool,
    pub jump_held: bool,
    pub dive: bool,
}

#[derive(Clone, Debug)]
pub struct Player {
    /// Centre of the feet; collision uses an upright capsule.
    pub pos: Vec3,
    pub velocity: Vec3,
    pub grounded: bool,
    pub ground_platform: Option<usize>,
    pub support_velocity: Vec3,
    pub air_carry: Vec3,
    pub crushed: bool,
    pub facing: f32,
    pub action: &'static str,
    pub wall_normal: Vec3,
    pub jumps: u32,
    pub wall_kicks: u32,
    pub long_jumps: u32,
    pub animation_phase: f32,
    pub motion: Move,
    pub motion_time: f32,
    pub dives: u32,
    pub rollouts: u32,
    pub landings: u32,
    pub impact_speed: f32,
    pub ground_normal: Vec3,
    pub peak_speed: f32,
    pub chain: u32,
    pub last_gain: f32,
    gain_time: f32,
    landing_grace: f32,
    dive_used: bool,
    coyote: f32,
    jump_buffer: f32,
    wall_grace: f32,
    kick_lock: f32,
    last_wall: Vec3,
    long_air: bool,
    jump_cuttable: bool,
    wall_contact_age: f32,
    touched_wall: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            pos: SPAWN,
            velocity: Vec3::ZERO,
            grounded: true,
            ground_platform: None,
            support_velocity: Vec3::ZERO,
            air_carry: Vec3::ZERO,
            crushed: false,
            facing: 0.,
            action: "READY",
            wall_normal: Vec3::ZERO,
            jumps: 0,
            wall_kicks: 0,
            long_jumps: 0,
            animation_phase: 0.,
            motion: Move::Normal,
            motion_time: 0.,
            dives: 0,
            rollouts: 0,
            landings: 0,
            impact_speed: 0.,
            ground_normal: Vec3::Y,
            peak_speed: 0.,
            chain: 0,
            last_gain: 0.,
            gain_time: 0.,
            landing_grace: 0.,
            dive_used: false,
            coyote: 0.1,
            jump_buffer: 0.,
            wall_grace: 0.,
            kick_lock: 0.,
            last_wall: Vec3::ZERO,
            long_air: false,
            jump_cuttable: false,
            wall_contact_age: 0.,
            touched_wall: false,
        }
    }
}

impl Player {
    pub fn respawn(&mut self, pos: Vec3) {
        let counts = (
            self.jumps,
            self.wall_kicks,
            self.long_jumps,
            self.dives,
            self.rollouts,
            self.landings,
        );
        *self = Self::default();
        self.pos = pos;
        self.grounded = false;
        self.coyote = 0.;
        (
            self.jumps,
            self.wall_kicks,
            self.long_jumps,
            self.dives,
            self.rollouts,
            self.landings,
        ) = counts;
    }

    pub fn speed(&self) -> f32 {
        Vec2::new(self.velocity.x, self.velocity.z).length()
    }

    pub fn height(&self) -> f32 {
        if matches!(self.motion, Move::Dive | Move::Slide) {
            0.72
        } else {
            HEIGHT
        }
    }
    pub fn interpolated(&self, previous: &Self, alpha: f32) -> Self {
        let alpha = alpha.clamp(0., 1.);
        let mut p = self.clone();
        p.pos = previous.pos.lerp(self.pos, alpha);
        p.facing = previous.facing + angle_delta(previous.facing, self.facing) * alpha;
        p.animation_phase =
            previous.animation_phase + (self.animation_phase - previous.animation_phase) * alpha;
        if previous.motion == self.motion {
            p.motion_time =
                previous.motion_time + (self.motion_time - previous.motion_time) * alpha;
        }
        p
    }
    fn can_stand(&self, world: &World) -> bool {
        !world
            .collision_solids()
            .any(|s| capsule_overlaps(self.pos, HEIGHT, s.min, s.max))
            && !world.ramps.iter().any(|r| {
                r.height(self.pos.x, self.pos.z)
                    .is_some_and(|h| self.pos.y < h - 0.01)
            })
    }
    pub fn step(&mut self, input: Input, yaw: f32, world: &World, dt: f32) {
        self.crushed = false;
        self.gain_time = (self.gain_time - dt).max(0.);
        self.landing_grace = (self.landing_grace - dt).max(0.);
        let retained_landing_grace = self.landing_grace;
        if self.grounded {
            self.ground_normal = world
                .ground_surface(self.pos)
                .filter(|s| (s.0 - self.pos.y).abs() < 0.3)
                .map_or(Vec3::Y, |s| s.1);
            if self.ground_normal.y < 0.9999 {
                let speed = self.velocity.length();
                let tangent =
                    self.velocity - self.ground_normal * self.velocity.dot(self.ground_normal);
                self.velocity = tangent.normalize_or_zero() * speed;
                let gravity = Vec3::NEG_Y * GRAVITY;
                self.velocity +=
                    (gravity - self.ground_normal * gravity.dot(self.ground_normal)) * dt;
            }
        }
        if self.grounded {
            if let Some(p) = self.ground_platform.and_then(|i| world.platforms.get(i)) {
                self.pos += p.delta;
                self.support_velocity = p.velocity;
                if p.delta.y > 0.
                    && world.collision_solids().any(|s| {
                        s.min.y > self.pos.y + RADIUS
                            && capsule_overlaps(self.pos, self.height(), s.min, s.max)
                    })
                {
                    self.crushed = true;
                    return;
                }
                self.move_horizontal(world, 0.);
                // A lift must not carry a capsule through a stationary roof.
                if !self.can_fit(world) {
                    self.crushed = true;
                    return;
                }
            } else {
                self.support_velocity = Vec3::ZERO;
            }
        }
        self.coyote = if self.grounded {
            0.10
        } else {
            (self.coyote - dt).max(0.)
        };
        self.jump_buffer = if input.jump {
            0.09
        } else {
            (self.jump_buffer - dt).max(0.)
        };
        self.wall_grace = (self.wall_grace - dt).max(0.);
        self.kick_lock = (self.kick_lock - dt).max(0.);
        self.touched_wall = false;
        self.motion_time += dt;
        let forward = vec3(-yaw.sin(), 0., -yaw.cos());
        let right = vec3(yaw.cos(), 0., -yaw.sin());
        let strength = input.movement.length().min(1.);
        let wish = (right * input.movement.x + forward * input.movement.y).normalize_or_zero();
        if wish.length_squared() > 0. && self.motion == Move::Normal {
            let target = (-wish.x).atan2(-wish.z);
            let difference = (target - self.facing + std::f32::consts::PI)
                .rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI;
            self.facing += difference.clamp(-18. * dt, 18. * dt);
        }
        // Keep the upward component earned on a slope when launching, so a
        // fast uphill long jump does not immediately collide with the hill.
        let slope_launch = if self.grounded && self.ground_normal != Vec3::Y {
            self.velocity.y.max(0.)
        } else {
            0.
        };
        // A dive can happen once per flight. Its landing becomes a short slide;
        // a fresh jump or dive press rolls out, including a buffered landing press.
        if self.motion == Move::Slide && !self.grounded {
            self.motion = Move::Dive;
            self.motion_time = 0.;
        }
        if self.motion == Move::Slide {
            if (self.jump_buffer > 0. || input.dive) && self.can_stand(world) {
                self.motion = Move::Rollout;
                self.motion_time = 0.;
                self.velocity.y = 7.6 + slope_launch;
                self.inherit_platform();
                let direction = Vec3::new(self.velocity.x, 0., self.velocity.z).normalize_or_zero();
                let direction = if direction == Vec3::ZERO {
                    vec3(-self.facing.sin(), 0., -self.facing.cos())
                } else {
                    direction
                };
                let speed = self.boost_speed(self.speed().max(8.), 3.8);
                self.velocity.x = direction.x * speed;
                self.velocity.z = direction.z * speed;
                self.grounded = false;
                self.coyote = 0.;
                self.jump_buffer = 0.;
                self.rollouts += 1;
            } else if ((self.motion_time > 0.55
                && !input.sprint
                && self.ground_normal.y >= WALKABLE_NORMAL_Y)
                || self.speed() < 2.)
                && self.can_stand(world)
            {
                self.motion = Move::Normal;
                self.dive_used = false;
            }
        } else if input.dive && self.motion == Move::Normal && !self.dive_used {
            let direction = if wish != Vec3::ZERO {
                wish
            } else {
                Vec3::new(self.velocity.x, 0., self.velocity.z).normalize_or_zero()
            };
            let direction = if direction == Vec3::ZERO {
                vec3(-self.facing.sin(), 0., -self.facing.cos())
            } else {
                direction
            };
            let direction = self.launch_direction(direction);
            let speed = self.boost_speed(self.speed().max(10.), 4.);
            self.velocity.x = direction.x * speed;
            self.velocity.z = direction.z * speed;
            self.velocity.y = if self.grounded {
                5.2 + slope_launch
            } else {
                (self.velocity.y + 2.).clamp(-10., 4.)
            };
            if self.grounded {
                self.inherit_platform();
            }
            self.facing += angle_delta(self.facing, (-direction.x).atan2(-direction.z));
            self.motion = Move::Dive;
            self.motion_time = 0.;
            self.dive_used = true;
            self.dives += 1;
            self.grounded = false;
            self.long_air = false;
            self.jump_cuttable = false;
            self.coyote = 0.;
            self.jump_buffer = 0.;
        }
        let special = self.motion != Move::Normal;
        let max_speed = if special {
            self.speed().max(8.)
        } else if self.long_air {
            15.0
        } else if input.sprint {
            SPRINT_SPEED
        } else {
            WALK_SPEED
        };
        let horizontal = vec3(self.velocity.x, 0., self.velocity.z);
        // How firmly the stick holds earned speed. Light input deliberately
        // brakes for precise landings and full input keeps momentum; partial
        // input blends between them rather than switching at a threshold.
        let hold = smoothstep(0.55, 0.9, strength);
        let accel = if self.grounded && self.ground_normal.y < WALKABLE_NORMAL_Y {
            if strength < 0.001 {
                1.5
            } else {
                7.
            }
        } else if special {
            7.
        } else if self.grounded {
            if strength < 0.001 {
                90.
            } else if horizontal.dot(wish) < -0.1 {
                110.
            } else {
                70.
            }
        } else if strength > 0.001 {
            let full = if self.long_air { 10. } else { 24. };
            36. + (full - 36.) * hold
        } else if self.long_air {
            10.
        } else {
            24.
        };
        if self.kick_lock <= 0. && self.motion != Move::Slide {
            let target = if special {
                let direction = vec3(self.velocity.x, 0., self.velocity.z).normalize_or_zero();
                (direction + wish * dt * 2.).normalize_or_zero() * max_speed
            } else {
                wish * max_speed * strength
            };
            let horizontal = vec3(self.velocity.x, 0., self.velocity.z);
            // Speed above the input's own target is kept in proportion to how
            // firmly, and how nearly along the motion, the stick is held.
            let keep = if !special && horizontal.length() > max_speed * strength + 0.1 {
                hold * smoothstep(-0.2, 0.15, horizontal.normalize_or_zero().dot(wish))
            } else {
                0.
            };
            let change = if wish == Vec3::ZERO && !self.grounded {
                Vec3::ZERO
            } else {
                let braking = (target - horizontal).clamp_length_max(accel * dt);
                if keep > 0. {
                    // Steer the velocity rather than replacing earned speed with a
                    // walking target. Faster travel needs a wider turning radius.
                    let rate = if self.grounded { 4.5 } else { 2.2 };
                    let steering = (steer(horizontal, wish, rate, dt) - horizontal)
                        .clamp_length_max(accel * dt);
                    steering * keep + braking * (1. - keep)
                } else {
                    braking
                }
            };
            self.velocity.x += change.x;
            self.velocity.z += change.z;
            if keep > 0. && self.grounded && self.landing_grace <= 0. {
                self.drag((0.15 + 0.0001 * self.speed().powi(2)) * dt * keep);
            }
            if strength > 0.001 && horizontal.dot(wish) < -0.1 {
                self.chain = 0;
            }
        }
        if special && self.speed() > 1. {
            let direction = vec3(self.velocity.x, 0., self.velocity.z);
            let target = (-direction.x).atan2(-direction.z);
            self.facing += angle_delta(self.facing, target).clamp(-8. * dt, 8. * dt);
        }
        if self.motion == Move::Slide {
            let horizontal = vec3(self.velocity.x, 0., self.velocity.z);
            let slowed = if !self.can_stand(world) && strength > 0. {
                horizontal + (wish * 3. * strength - horizontal).clamp_length_max(20. * dt)
            } else {
                // Belly slides carve gentle arcs, wider at speed, so long
                // downhill slides held with Sprint can still be steered.
                let carved = steer(horizontal, wish, 1.6 * strength, dt);
                carved - carved.clamp_length_max(2.8 * dt)
            };
            self.velocity.x = slowed.x;
            self.velocity.z = slowed.z;
        }
        if self.motion == Move::Normal && self.jump_buffer > 0. && self.coyote > 0. {
            self.velocity.y = if input.sprint && self.speed() > 3. {
                9.0 + slope_launch
            } else {
                11.2 + slope_launch
            };
            self.long_air = input.sprint && self.speed() > 3.;
            self.jump_cuttable = !self.long_air;
            if self.long_air {
                let launch = if wish.length_squared() > 0. {
                    wish
                } else {
                    vec3(self.velocity.x, 0., self.velocity.z).normalize_or_zero()
                };
                let launch = self.launch_direction(launch);
                let speed = self.boost_speed(self.speed().max(10.5), 6.);
                self.velocity.x = launch.x * speed;
                self.velocity.z = launch.z * speed;
                self.long_jumps += 1;
                self.action = "LONG JUMP";
            } else {
                self.action = "JUMP";
            }
            self.inherit_platform();
            self.jumps += 1;
            self.grounded = false;
            self.coyote = 0.;
            self.jump_buffer = 0.;
        } else if self.motion == Move::Normal
            && self.jump_buffer > 0.
            && !self.grounded
            && self.wall_grace > 0.
            && self.wall_contact_age <= 0.12
            && self.kick_lock <= 0.
            && self.wall_normal.dot(self.last_wall) < 0.8
        {
            let tangent = wish - self.wall_normal * wish.dot(self.wall_normal);
            let retained = self.velocity - self.wall_normal * self.velocity.dot(self.wall_normal);
            self.velocity =
                self.wall_normal * 11.5 + vec3(retained.x, 11.5, retained.z) + tangent * 3.;
            self.air_carry = Vec3::ZERO;
            self.ground_platform = None;
            self.last_wall = self.wall_normal;
            self.kick_lock = 0.10;
            self.jump_buffer = 0.;
            self.wall_grace = 0.;
            self.long_air = false;
            self.jump_cuttable = false;
            self.wall_contact_age = 0.;
            self.wall_kicks += 1;
            self.action = "WALL KICK";
        }
        let old_pos = self.pos;
        let old_y = old_pos.y;
        let was_grounded = self.grounded;
        let followed_surface = was_grounded
            && world
                .continuous_surface(old_pos.x, old_pos.z)
                .is_some_and(|s| (s.0 - old_y).abs() < 0.02);
        self.move_horizontal(world, dt);
        // Climb the continuous ramp surface, but never snap an airborne player up.
        if followed_surface {
            if let Some((h, n)) = world.continuous_surface(self.pos.x, self.pos.z) {
                let reach = 0.16 + self.speed() * dt * 1.5;
                if (h - old_y).abs() <= reach && self.pos.y <= old_y + reach {
                    self.pos.y = h;
                    self.ground_normal = n;
                }
            }
        }
        let before_y = self.pos.y;
        let gravity =
            if self.velocity.y > 0. && self.jump_cuttable && !input.jump_held && !input.jump {
                GRAVITY * 2.8
            } else if self.velocity.y < 0. {
                GRAVITY * 1.25
            } else {
                GRAVITY
            };
        let ground_vertical = self.velocity.y;
        self.velocity.y = if was_grounded {
            -GRAVITY * dt
        } else {
            (self.velocity.y - gravity * dt).max(-62.)
        };
        self.pos.y += self.velocity.y * dt;
        self.grounded = false;
        let carried_platform = self.ground_platform;
        self.ground_platform = None;
        let falling_speed = self.velocity.y;
        for (index, s) in world.collision_solids().enumerate() {
            let gap = horizontal_gap(self.pos, s.min, s.max).length_squared();
            if gap >= RADIUS * RADIUS {
                continue;
            }
            let round = (RADIUS * RADIUS - gap).sqrt();
            let top = s.max.y - RADIUS + round;
            let ceiling = s.min.y - self.height() + RADIUS - round;
            let platform = index.checked_sub(world.solids.len());
            let mover = platform.and_then(|i| world.platforms.get(i));
            let riding = was_grounded && carried_platform == platform && platform.is_some();
            let relative_y =
                falling_speed - mover.map_or(0., |p| if riding { 0. } else { p.velocity.y });
            let previous_top = top - mover.map_or(0., |p| if riding { 0. } else { p.delta.y });
            if relative_y <= 0. && before_y >= previous_top - 0.02 && self.pos.y <= top {
                self.pos.y = top;
                self.land();
                self.ground_platform = platform;
                self.support_velocity = mover.map_or(Vec3::ZERO, |p| p.velocity);
            } else if self.velocity.y > 0. && before_y <= ceiling + 0.01 && self.pos.y >= ceiling {
                self.pos.y = ceiling;
                self.velocity.y = 0.;
            }
        }
        for r in &world.ramps {
            if let Some(h) = r.height(self.pos.x, self.pos.z) {
                if self.velocity.y <= 0. && before_y >= h - 0.15 && self.pos.y <= h {
                    self.pos.y = h;
                    self.land();
                    self.ground_normal = r.normal();
                }
            }
        }
        if let Some((h, n)) = world
            .terrain
            .as_ref()
            .and_then(|t| t.sample(self.pos.x, self.pos.z))
        {
            if self.pos.y < h && (before_y >= h - 0.2 || was_grounded) {
                self.pos.y = h;
                self.land();
                self.ground_normal = n;
            }
        }
        if self.grounded {
            if was_grounded {
                self.landing_grace = retained_landing_grace;
            }
            self.velocity.y = -(self.ground_normal.x * self.velocity.x
                + self.ground_normal.z * self.velocity.z)
                / self.ground_normal.y.max(0.1);
        } else if was_grounded {
            // The slope's upward velocity launches naturally over a crest.
            self.velocity.y = ground_vertical - GRAVITY * dt;
        }
        if was_grounded && !self.grounded {
            // Walking off inherits the same motion as jumping; coyote jumps use
            // the retained vertical support speed without adding it twice.
            self.air_carry = Vec3::new(self.support_velocity.x, 0., self.support_velocity.z);
            self.velocity.y += self.support_velocity.y;
        }
        if self.grounded && !self.can_fit(world) {
            self.crushed = true;
        }
        if self.touched_wall {
            self.wall_contact_age += dt;
        } else if self.wall_grace <= 0. {
            self.wall_contact_age = 0.;
        }
        if !self.grounded
            && self.touched_wall
            && self.wall_contact_age <= 0.12
            && self.velocity.y < -5.5
        {
            self.velocity.y = -5.5;
            self.action = "WALL SLIDE";
        }
        if self.grounded && self.motion == Move::Normal {
            self.dive_used = false;
            if self.ground_normal.y < WALKABLE_NORMAL_Y {
                self.action = "SLOPE SLIDE";
            }
        }
        if self.motion == Move::Rollout && self.motion_time >= 0.45 {
            self.motion = Move::Normal;
        }
        if self.motion != Move::Normal {
            self.action = match self.motion {
                Move::Dive => "DIVE",
                Move::Slide => "DIVE SLIDE",
                Move::Rollout => "ROLLOUT",
                Move::Normal => unreachable!(),
            };
        } else if self.grounded {
            self.action = if self.ground_normal.y < WALKABLE_NORMAL_Y {
                "SLOPE SLIDE"
            } else if self.speed() > 0.5 {
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
        // About 6 footfalls a second at sprint speed. Above 14 m/s the cadence
        // holds and strides lengthen, so legs and footsteps never blur.
        self.animation_phase += self.speed().min(14.) * dt * 1.9;
        self.peak_speed = self.peak_speed.max(self.speed());
        if self.gain_time <= 0. && self.grounded && self.motion == Move::Normal {
            self.chain = 0;
        }
    }

    fn can_fit(&self, world: &World) -> bool {
        !world
            .collision_solids()
            .any(|s| capsule_overlaps(self.pos, self.height(), s.min, s.max))
    }
    fn inherit_platform(&mut self) {
        self.air_carry = Vec3::new(self.support_velocity.x, 0., self.support_velocity.z);
        self.velocity.y += self.support_velocity.y;
        self.ground_platform = None;
        self.support_velocity = Vec3::ZERO;
    }
    fn land(&mut self) {
        if self.velocity.y < -1. {
            self.landings += 1;
            self.impact_speed = -self.velocity.y;
        }
        if self.motion == Move::Dive {
            self.motion = Move::Slide;
            self.motion_time = 0.;
        } else if self.motion == Move::Rollout {
            // A rollout that meets raised ground ends there. Staying in the
            // airborne state would block jumps and let buffered presses expire.
            self.motion = Move::Normal;
        }
        self.grounded = true;
        self.ground_normal = Vec3::Y;
        self.landing_grace = 0.10;
        self.air_carry = Vec3::ZERO;
        self.ground_platform = None;
        self.support_velocity = Vec3::ZERO;
        self.velocity.y = 0.;
        self.long_air = false;
        self.last_wall = Vec3::ZERO;
        self.wall_grace = 0.;
        self.jump_cuttable = false;
        self.wall_contact_age = 0.;
    }

    fn boost_speed(&mut self, speed: f32, impulse: f32) -> f32 {
        self.last_gain = momentum_gain(speed, impulse);
        self.chain += 1;
        self.gain_time = 1.5;
        speed + self.last_gain
    }
    fn launch_direction(&self, wish: Vec3) -> Vec3 {
        let horizontal = vec3(self.velocity.x, 0., self.velocity.z);
        if horizontal.length() < 12. {
            wish.normalize_or_zero()
        } else {
            steer(horizontal, wish, 2., 0.12).normalize_or_zero()
        }
    }
    fn drag(&mut self, amount: f32) {
        let h = vec3(self.velocity.x, 0., self.velocity.z);
        let v = h - h.clamp_length_max(amount);
        self.velocity.x = v.x;
        self.velocity.z = v.z;
    }

    fn contact(&mut self, normal: Vec3) {
        self.wall_normal = normal;
        self.wall_grace = WALL_GRACE;
        self.touched_wall = true;
        let carry_into = self.air_carry.dot(normal);
        if carry_into < 0. {
            self.air_carry -= normal * carry_into;
        }
        let into = self.velocity.dot(normal);
        if into < 0. {
            self.velocity -= normal * into;
        }
    }
    fn move_horizontal(&mut self, world: &World, dt: f32) {
        let carry = if self.grounded {
            Vec3::ZERO
        } else {
            self.air_carry
        };
        let delta = (vec3(self.velocity.x, 0., self.velocity.z) + carry) * dt;
        let steps = (delta.length() / (RADIUS * 0.5)).ceil().max(1.) as usize;
        for step in 0..steps {
            let previous = self.pos;
            self.pos += (vec3(self.velocity.x, 0., self.velocity.z)
                + if self.grounded {
                    Vec3::ZERO
                } else {
                    self.air_carry
                })
                * (dt / steps as f32);
            // Horizontal and vertical resolution are separate. Test rising
            // launches at their height along the swept path, not at the old
            // feet height, which would falsely ground an uphill jump.
            let path_y = self.pos.y
                + if self.grounded {
                    0.
                } else {
                    self.velocity.y.max(0.) * dt * (step + 1) as f32 / steps as f32
                };
            if let Some((h, n)) = world.continuous_surface(self.pos.x, self.pos.z) {
                let follows = self.grounded
                    && world
                        .continuous_surface(previous.x, previous.z)
                        .is_some_and(|s| (s.0 - previous.y).abs() < 0.02);
                if follows && (h - previous.y).abs() <= 0.28 {
                    self.pos.y = h;
                    self.ground_normal = n;
                } else if path_y < h && path_y + self.height() > h {
                    // Sweep every short horizontal segment against the field;
                    // fast travel must not tunnel into an uphill face.
                    let penetration = (h - path_y) * n.y;
                    self.pos += n * penetration;
                    let into = self.velocity.dot(n);
                    if into < 0. {
                        self.velocity -= n * into;
                    }
                    if n.y >= WALKABLE_NORMAL_Y {
                        self.pos.y = h;
                        self.land();
                        self.ground_normal = n;
                    }
                }
            }
            for _ in 0..3 {
                for s in world.collision_solids() {
                    if self.pos.y >= s.max.y - 0.001
                        || self.pos.y + self.height() <= s.min.y + 0.001
                    {
                        continue;
                    }
                    if !capsule_overlaps(self.pos, self.height(), s.min, s.max) {
                        continue;
                    }
                    if self.grounded && s.max.y - self.pos.y <= 0.25 {
                        let raised = vec3(self.pos.x, s.max.y, self.pos.z);
                        if !world.collision_solids().any(|other| {
                            capsule_overlaps(raised, self.height(), other.min, other.max)
                        }) {
                            self.pos.y = s.max.y;
                            continue;
                        }
                    }
                    let vertical_gap = (s.min.y - (self.pos.y + self.height() - RADIUS))
                        .max(self.pos.y + RADIUS - s.max.y)
                        .max(0.);
                    let radius = (RADIUS * RADIUS - vertical_gap * vertical_gap)
                        .max(0.)
                        .sqrt();
                    self.push_circle(s.min, s.max, radius);
                }
                for r in &world.ramps {
                    let x = self.pos.x.clamp(r.min.x, r.max.x);
                    let z = self.pos.z.clamp(r.min.z, r.max.z);
                    if horizontal_gap(self.pos, r.min, r.max).length_squared() < RADIUS * RADIUS
                        && path_y + self.height() > r.base
                        && r.height(x, z).is_some_and(|h| path_y + 0.25 < h)
                    {
                        self.push_circle(r.min, r.max, RADIUS);
                    }
                }
            }
        }
    }
    fn push_circle(&mut self, min: Vec3, max: Vec3, radius: f32) {
        let gap = horizontal_gap(self.pos, min, max);
        let distance = gap.length();
        if distance >= radius - 0.00001 {
            return;
        }
        let (normal, penetration) = if distance > 0.00001 {
            (
                vec3(gap.x / distance, 0., gap.y / distance),
                radius - distance,
            )
        } else {
            let candidates = [
                (self.pos.x - min.x, Vec3::NEG_X),
                (max.x - self.pos.x, Vec3::X),
                (self.pos.z - min.z, Vec3::NEG_Z),
                (max.z - self.pos.z, Vec3::Z),
            ];
            let (gap, normal) = candidates
                .into_iter()
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .unwrap();
            (normal, gap + radius)
        };
        self.pos += normal * (penetration + 0.00001);
        self.contact(normal);
    }
}
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}
fn steer(horizontal: Vec3, wish: Vec3, rate: f32, dt: f32) -> Vec3 {
    let speed = horizontal.length();
    if speed < 0.001 || wish == Vec3::ZERO {
        return horizontal;
    }
    let angle = horizontal.x.atan2(horizontal.z);
    let target = wish.x.atan2(wish.z);
    let turn = angle_delta(angle, target).clamp(
        -rate * dt / (1. + speed / 28.),
        rate * dt / (1. + speed / 28.),
    );
    vec3(
        (angle + turn).sin() * speed,
        0.,
        (angle + turn).cos() * speed,
    )
}
fn horizontal_gap(p: Vec3, min: Vec3, max: Vec3) -> Vec2 {
    Vec2::new(p.x - p.x.clamp(min.x, max.x), p.z - p.z.clamp(min.z, max.z))
}
fn capsule_overlaps(p: Vec3, height: f32, min: Vec3, max: Vec3) -> bool {
    let y = (min.y - (p.y + height - RADIUS))
        .max(p.y + RADIUS - max.y)
        .max(0.);
    horizontal_gap(p, min, max).length_squared() + y * y < RADIUS * RADIUS - 0.000001
}
