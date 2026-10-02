use crate::world::World;
use glam::{vec3, Mat4, Vec3};

/// Furthest the smoothed look target may trail the player vertically.
pub const MAX_VERTICAL_LAG: f32 = 2.5;

#[derive(Clone)]
pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target: Vec3,
    pub eye: Vec3,
    boom: f32,
    manual_time: f32,
    speed_blend: f32,
}

pub fn angle_delta(from: f32, to: f32) -> f32 {
    (to - from + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}
impl Camera {
    pub fn new(player: Vec3) -> Self {
        let mut c = Self {
            yaw: 0.,
            pitch: 0.43,
            distance: 10.,
            target: player + Vec3::Y * 1.1,
            eye: Vec3::ZERO,
            boom: 10.,
            manual_time: 0.,
            speed_blend: 0.,
        };
        c.eye = c.target + c.direction() * c.boom;
        c
    }
    fn direction(&self) -> Vec3 {
        vec3(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        )
    }
    pub fn orbit(&mut self, yaw: f32, pitch: f32) {
        self.yaw += yaw;
        self.pitch += pitch;
        self.manual_time = 1.5;
    }
    pub fn recenter(&mut self, facing: f32) {
        self.yaw += angle_delta(self.yaw, facing);
        self.manual_time = 1.5;
    }
    pub fn follow(
        &mut self,
        player: &crate::engine::physics::Player,
        auto_align: bool,
        world: &World,
        dt: f32,
    ) {
        self.manual_time = (self.manual_time - dt).max(0.);
        let goal = ((player.speed() - 12.) / 45.).clamp(0., 1.);
        self.speed_blend += (goal - self.speed_blend) * (1. - (-3. * dt).exp());
        if auto_align && player.grounded && player.speed() > 3. && self.manual_time <= 0. {
            self.yaw += angle_delta(self.yaw, player.facing) * (1. - (-1.6 * dt).exp());
        }
        self.update_target(
            player.pos,
            player.grounded,
            (player.height() * 0.8).min(1.1),
            world,
            dt,
        );
    }
    pub fn update(&mut self, player: Vec3, world: &World, dt: f32) {
        self.update_target(player, true, 1.1, world, dt);
    }
    fn update_target(
        &mut self,
        player: Vec3,
        grounded: bool,
        head_height: f32,
        world: &World,
        dt: f32,
    ) {
        self.pitch = self.pitch.clamp(0.08, 1.25);
        self.distance = self.distance.clamp(4., 17.);
        let goal = player + Vec3::Y * head_height;
        let horizontal = 1. - (-14. * dt).exp();
        let vertical = 1. - (-(if grounded { 10. } else { 6. }) * dt).exp();
        self.target.x += (goal.x - self.target.x) * horizontal;
        self.target.z += (goal.z - self.target.z) * horizontal;
        self.target.y += (goal.y - self.target.y) * vertical;
        // Smoothing alone trails a terminal-velocity fall by about 10 m, which
        // drops the player out of frame. Keep the lag within a fixed band.
        self.target.y = self
            .target
            .y
            .clamp(goal.y - MAX_VERTICAL_LAG, goal.y + MAX_VERTICAL_LAG);
        if world.solids.iter().any(|s| s.contains(self.target, 0.27)) {
            self.target = goal;
        }
        let direction = self.direction();
        let distance = self.distance + self.speed_blend * 4.;
        let offset = direction * distance;
        let mut fraction = 1_f32;
        // Sweep a padded camera boom rather than sampling block corners. Retract
        // immediately to avoid clipping; recover distance smoothly when clear.
        for s in world.collision_solids() {
            if let Some(hit) = ray_box(
                self.target,
                offset,
                s.min - Vec3::splat(0.28),
                s.max + Vec3::splat(0.28),
            ) {
                fraction = fraction.min((hit - 0.02 / self.distance).max(0.));
            }
        }
        for i in 1..=100 {
            let t = i as f32 / 100.;
            let p = self.target + offset * t;
            if world.ramps.iter().any(|r| {
                r.height(p.x, p.z)
                    .is_some_and(|h| p.y <= h + 0.28 && p.y >= r.base - 0.28)
            }) || world
                .terrain
                .as_ref()
                .and_then(|field| field.sample(p.x, p.z))
                .is_some_and(|s| p.y <= s.0 + 0.28)
            {
                fraction = fraction.min((i - 1) as f32 / 100.);
                break;
            }
        }
        let safe = (distance * fraction).max(0.05);
        self.boom = if safe < self.boom {
            safe
        } else {
            self.boom + (safe - self.boom) * (1. - (-5. * dt).exp())
        };
        self.eye = self.target + direction * self.boom;
    }
    pub fn interpolated(&self, previous: &Self, alpha: f32) -> Self {
        let mut c = self.clone();
        c.eye = previous.eye.lerp(self.eye, alpha.clamp(0., 1.));
        c.target = previous.target.lerp(self.target, alpha.clamp(0., 1.));
        c
    }
    pub fn matrix(&self, aspect: f32) -> Mat4 {
        // Overview cameras have no nearby geometry: move their near plane out
        // to preserve depth precision across the larger landscape.
        let near = (self.eye.distance(self.target) * 0.012).clamp(0.12, 8.);
        Mat4::perspective_rh_gl(
            (58. + self.speed_blend * 12.).to_radians(),
            aspect,
            near,
            2500.,
        ) * Mat4::look_at_rh(self.eye, self.target, Vec3::Y)
    }
}
fn ray_box(start: Vec3, delta: Vec3, min: Vec3, max: Vec3) -> Option<f32> {
    let (mut entry, mut exit) = (0_f32, 1_f32);
    for axis in 0..3 {
        if delta[axis].abs() < 1e-6 {
            if start[axis] < min[axis] || start[axis] > max[axis] {
                return None;
            }
        } else {
            let a = (min[axis] - start[axis]) / delta[axis];
            let b = (max[axis] - start[axis]) / delta[axis];
            entry = entry.max(a.min(b));
            exit = exit.min(a.max(b));
            if entry > exit {
                return None;
            }
        }
    }
    (exit >= 0. && entry <= 1.).then_some(entry)
}
