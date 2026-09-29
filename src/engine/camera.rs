use crate::world::World;
use glam::{vec3, Mat4, Vec3};

pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target: Vec3,
    pub eye: Vec3,
}

impl Camera {
    pub fn new(player: Vec3) -> Self {
        let mut c = Self {
            yaw: 0.,
            pitch: 0.43,
            distance: 10.,
            target: player + Vec3::Y,
            eye: Vec3::ZERO,
        };
        c.eye = c.target + c.offset();
        c
    }
    fn offset(&self) -> Vec3 {
        vec3(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        ) * self.distance
    }
    pub fn update(&mut self, player: Vec3, world: &World, dt: f32) {
        self.pitch = self.pitch.clamp(0.12, 1.2);
        self.distance = self.distance.clamp(4., 17.);
        self.target = self
            .target
            .lerp(player + Vec3::Y * 1.1, 1. - (-12. * dt).exp());
        let offset = self.offset();
        let mut fraction = 1.;
        // Sample the camera boom against the same solid geometry as the player.
        for i in 1..=80 {
            let t = i as f32 / 80.;
            let p = self.target + offset * t;
            if world.solids.iter().any(|s| s.contains(p, 0.20))
                || world.ramps.iter().any(|r| {
                    r.height(p.x, p.z)
                        .is_some_and(|h| p.y <= h + 0.2 && p.y >= r.min.y)
                })
            {
                fraction = ((i - 1) as f32 / 80.).max(0.07);
                break;
            }
        }
        self.eye = self.target + offset * fraction;
    }
    pub fn matrix(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_rh_gl(58_f32.to_radians(), aspect, 0.08, 180.)
            * Mat4::look_at_rh(self.eye, self.target, Vec3::Y)
    }
}
