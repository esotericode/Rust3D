use glam::{vec3, Vec3};

pub const NAVY: [f32; 3] = [0.18, 0.25, 0.31];
pub const MINT: [f32; 3] = [0.25, 0.88, 0.76];
pub const ORANGE: [f32; 3] = [1.0, 0.48, 0.25];
pub const CONCRETE: [f32; 3] = [0.69, 0.73, 0.71];
pub const SPAWN: Vec3 = Vec3::new(0.0, 0.0, 13.0);

#[derive(Clone, Debug)]
pub struct Solid {
    pub min: Vec3,
    pub max: Vec3,
    pub color: [f32; 3],
}

impl Solid {
    pub fn new(center: Vec3, size: Vec3, color: [f32; 3]) -> Self {
        Self {
            min: center - size * 0.5,
            max: center + size * 0.5,
            color,
        }
    }
    pub fn contains(&self, p: Vec3, padding: f32) -> bool {
        p.cmpge(self.min - Vec3::splat(padding)).all()
            && p.cmple(self.max + Vec3::splat(padding)).all()
    }
}

/// A solid wedge rising toward negative Z. Its visible and collision surfaces
/// both use this exact height function.
#[derive(Clone, Debug)]
pub struct Ramp {
    pub min: Vec3,
    pub max: Vec3,
}
impl Ramp {
    pub fn height(&self, x: f32, z: f32) -> Option<f32> {
        if x < self.min.x || x > self.max.x || z < self.min.z || z > self.max.z {
            return None;
        }
        Some(self.min.y + (self.max.z - z) / (self.max.z - self.min.z) * (self.max.y - self.min.y))
    }
}

#[derive(Clone, Debug)]
pub struct Beacon {
    pub pos: Vec3,
    pub name: &'static str,
}

#[derive(Clone, Debug)]
pub struct World {
    pub solids: Vec<Solid>,
    pub ramps: Vec<Ramp>,
    pub beacons: Vec<Beacon>,
}

impl Default for World {
    fn default() -> Self {
        let mut solids = vec![
            Solid::new(vec3(0., -0.6, -3.), vec3(48., 1.2, 46.), CONCRETE),
            // Ramp landing.
            Solid::new(vec3(-8., 1., -1.), vec3(6., 2., 4.), NAVY),
            // Jump staircase.
            Solid::new(vec3(-9., 0.6, -8.), vec3(3., 1.2, 3.), NAVY),
            Solid::new(vec3(-13., 1.2, -11.), vec3(3., 2.4, 3.), NAVY),
            Solid::new(vec3(-9., 1.8, -15.), vec3(4., 3.6, 4.), NAVY),
            // Elevated launch and landing. Six metres of clear air.
            Solid::new(vec3(0., 1.5, -16.), vec3(5., 3., 7.), NAVY),
            Solid::new(vec3(10., 1.5, -16.), vec3(3., 3., 7.), NAVY),
            // Wall kick lane: 2.8 m clear width, open at both ends.
            Solid::new(vec3(15.2, 3., -3.), vec3(1.2, 6., 12.), NAVY),
            Solid::new(vec3(19.2, 3., -3.), vec3(1.2, 6., 12.), NAVY),
            Solid::new(vec3(17.2, 2.2, -9.5), vec3(5.2, 4.4, 2.), NAVY),
            // A low tunnel tests head collision.
            Solid::new(vec3(6., 1.2, 5.), vec3(0.7, 2.4, 5.), NAVY),
            Solid::new(vec3(10., 1.2, 5.), vec3(0.7, 2.4, 5.), NAVY),
            Solid::new(vec3(8., 2.6, 5.), vec3(4.7, 0.8, 5.), NAVY),
        ];
        // Small edge posts leave the playground open for fall/respawn tests.
        for x in [-23., 23.] {
            for z in [-25., -13., -1., 11., 19.] {
                solids.push(Solid::new(vec3(x, 0.55, z), vec3(0.5, 1.1, 0.5), NAVY));
            }
        }
        Self {
            solids,
            ramps: vec![
                Ramp {
                    min: vec3(-11., 0., 1.),
                    max: vec3(-5., 2., 9.),
                },
                Ramp {
                    min: vec3(-2.5, 0., -12.5),
                    max: vec3(2.5, 3., -3.5),
                },
            ],
            beacons: vec![
                Beacon {
                    pos: vec3(-8., 2., -1.),
                    name: "RAMP / FIND YOUR FEET",
                },
                Beacon {
                    pos: vec3(-9., 3.6, -15.),
                    name: "BLOCKS / JUMP THE STEPS",
                },
                Beacon {
                    pos: vec3(10., 3., -16.),
                    name: "GAP / SHIFT + SPACE",
                },
                Beacon {
                    pos: vec3(17.2, 4.4, -9.5),
                    name: "WALLS / KICK YOUR WAY UP",
                },
                Beacon {
                    pos: vec3(8., 0., 6.),
                    name: "TUNNEL / THE HOME STRETCH",
                },
            ],
        }
    }
}

impl World {
    pub fn floor_height(&self, p: Vec3) -> Option<f32> {
        let blocks = self
            .solids
            .iter()
            .filter(|s| {
                p.x >= s.min.x
                    && p.x <= s.max.x
                    && p.z >= s.min.z
                    && p.z <= s.max.z
                    && s.max.y <= p.y + 0.15
            })
            .map(|s| s.max.y);
        let ramps = self
            .ramps
            .iter()
            .filter_map(|r| r.height(p.x, p.z))
            .filter(|h| *h <= p.y + 0.15);
        blocks.chain(ramps).max_by(f32::total_cmp)
    }
}
