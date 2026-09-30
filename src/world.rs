use crate::course::{Course, MovingPlatform};
use crate::engine::terrain::{Seeded, Terrain, MAP_MIN, MAP_SIZE, REGIONS, SEED};
use glam::{vec3, Vec3};

pub const NAVY: [f32; 3] = [0.18, 0.25, 0.31];
pub const MINT: [f32; 3] = [0.25, 0.88, 0.76];
pub const ORANGE: [f32; 3] = [1.0, 0.48, 0.25];
pub const CONCRETE: [f32; 3] = [0.69, 0.73, 0.71];
pub const SPAWN: Vec3 = Vec3::new(0.0, 0.0, 13.0);
pub const LEVEL_SIZE: Vec3 = Vec3::new(360., 1.2, 420.);
pub const LEVEL_CENTER: Vec3 = Vec3::new(100., -0.6, -25.);
pub const TOWER: Vec3 = Vec3::new(-34., 0., -40.);

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
    pub fn normal(&self) -> Vec3 {
        vec3(
            0.,
            1.,
            (self.max.y - self.min.y) / (self.max.z - self.min.z),
        )
        .normalize()
    }
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
    pub terrain: Option<Terrain>,
    pub solids: Vec<Solid>,
    pub ramps: Vec<Ramp>,
    pub beacons: Vec<Beacon>,
    pub platforms: Vec<MovingPlatform>,
    pub course: Course,
    pub time: f64,
}

impl Default for World {
    fn default() -> Self {
        let mut solids = vec![
            Solid::new(LEVEL_CENTER, LEVEL_SIZE, CONCRETE),
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
        for x in [-55., 55.] {
            for z in [-67., -45., -23., -1., 21., 43.] {
                solids.push(Solid::new(vec3(x, 0.55, z), vec3(0.5, 1.1, 0.5), NAVY));
            }
        }
        // Thirteen ledges spiral around the core with 1.1 m rises. The final
        // ledge meets roof height so the finish avoids an abrupt reverse jump.
        solids.push(Solid::new(
            TOWER + vec3(0., 7.15, 0.),
            vec3(5., 14.3, 5.),
            NAVY,
        ));
        let ledges = [
            (0., 4.5),
            (4.5, 4.5),
            (4.5, 0.),
            (4.5, -4.5),
            (0., -4.5),
            (-4.5, -4.5),
            (-4.5, 0.),
            (-4.5, 4.5),
        ];
        for i in 0..13 {
            let (x, z) = ledges[i % ledges.len()];
            let y = (i + 1) as f32 * 1.1;
            solids.push(Solid::new(
                TOWER + vec3(x, y - 0.225, z),
                vec3(3.4, 0.45, 3.4),
                NAVY,
            ));
        }
        solids.push(Solid::new(vec3(34., 2., -22.), vec3(12., 4., 8.), NAVY));
        solids.push(Solid::new(vec3(-40., 1.5, -32.), vec3(10., 3., 6.), NAVY));
        for (i, x) in [-38., -28., 28., 40.].into_iter().enumerate() {
            for z in [21., 32.] {
                let height = 0.8 + i as f32 * 0.45;
                solids.push(Solid::new(
                    vec3(x, height * 0.5, z),
                    vec3(4., height, 4.),
                    NAVY,
                ));
            }
        }
        // Short wall-kick pairs and a broad open running area surround the lab.
        for x in [-17., -13.] {
            solids.push(Solid::new(vec3(x, 2.5, 28.), vec3(1., 5., 8.), NAVY));
        }
        let mut world = Self {
            terrain: Some(Terrain::highlands()),
            solids,
            platforms: Vec::new(),
            course: Course::default(),
            time: 0.,
            ramps: vec![
                Ramp {
                    min: vec3(-11., 0., 1.),
                    max: vec3(-5., 2., 9.),
                },
                Ramp {
                    min: vec3(-2.5, 0., -12.5),
                    max: vec3(2.5, 3., -3.5),
                },
                Ramp {
                    min: vec3(28., 0., -18.),
                    max: vec3(40., 4., 2.),
                },
                Ramp {
                    min: vec3(-45., 0., -29.),
                    max: vec3(-35., 3., -14.),
                },
                Ramp {
                    min: vec3(25., 0., 15.),
                    max: vec3(31., 2., 27.),
                },
                Ramp {
                    min: vec3(-48., 0., 15.),
                    max: vec3(-42., 2.5, 29.),
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
                    name: "TUNNEL / THEN FIND THE TOWER",
                },
                Beacon {
                    pos: TOWER + Vec3::Y * 14.3,
                    name: "TOWER / CLIMB THE SPIRAL",
                },
                Beacon {
                    pos: vec3(34., 4., -22.),
                    name: "RIDGE / THE FINAL RUN",
                },
            ],
        };
        world.ramps.push(Ramp {
            min: vec3(48., 0., 36.),
            max: vec3(64., 8., 61.),
        });
        // Broad ramp-and-block practice gardens occupy the newly opened yard,
        // clear of the overhead course and its fall corridors.
        for (x, z, h) in [
            (90., -80., 3.),
            (130., -105., 4.),
            (175., -70., 5.),
            (215., -120., 6.),
        ] {
            world.ramps.push(Ramp {
                min: vec3(x - 4., 0., z),
                max: vec3(x + 4., h, z + 20.),
            });
            world
                .solids
                .push(Solid::new(vec3(x, h * 0.5, z - 4.), vec3(8., h, 8.), NAVY));
            for j in 0..4 {
                let height = 0.8 + j as f32 * 0.8;
                world.solids.push(Solid::new(
                    vec3(x + 14. + j as f32 * 5., height * 0.5, z + 7.),
                    vec3(3.5, height, 3.5),
                    NAVY,
                ));
            }
        }
        Course::build(&mut world);
        world.scatter_highlands();
        world
    }
}

impl World {
    fn scatter_highlands(&mut self) {
        let mut rng = Seeded(SEED);
        let mut blocks = 0;
        let mut ramps = 0;
        for _ in 0..4000 {
            let x = rng.range(MAP_MIN.x + 35., MAP_MIN.x + MAP_SIZE.x - 35.);
            let z = rng.range(MAP_MIN.y + 35., MAP_MIN.y + MAP_SIZE.y - 35.);
            if (x > -120. && x < 320. && z > -275. && z < 225.)
                || ((x - 100.).abs() < 45. && z > 190.)
                || REGIONS
                    .iter()
                    .any(|r| glam::Vec2::new(x - r.point.x, z - r.point.y).length() < 40.)
            {
                continue;
            }
            let t = self.terrain.as_ref().unwrap();
            let (h, n) = t.sample(x, z).unwrap();
            if n.y < 0.78 {
                continue;
            }
            if blocks < 180 {
                let sx = rng.range(1.5, 17.);
                let sz = rng.range(1.5, 17.);
                let height = rng.range(1., 18.);
                let mut low = h;
                let mut high = h;
                for dx in [-0.5, 0., 0.5] {
                    for dz in [-0.5, 0., 0.5] {
                        let y = t.sample(x + sx * dx, z + sz * dz).unwrap().0;
                        low = low.min(y);
                        high = high.max(y);
                    }
                }
                let bottom = low - 0.4;
                let top = high + height;
                let color = if blocks % 3 == 0 {
                    [0.46, 0.49, 0.48]
                } else {
                    NAVY
                };
                self.solids.push(Solid::new(
                    vec3(x, (top + bottom) * 0.5, z),
                    vec3(sx, top - bottom, sz),
                    color,
                ));
                blocks += 1;
            } else if ramps < 40 {
                let width = rng.range(6., 15.);
                let length = rng.range(18., 38.);
                let low = t.sample(x, z + length * 0.5).unwrap().0;
                let high = t.sample(x, z - length * 0.5).unwrap().0.max(low) + rng.range(4., 10.);
                self.ramps.push(Ramp {
                    min: vec3(x - width * 0.5, low, z - length * 0.5),
                    max: vec3(x + width * 0.5, high, z + length * 0.5),
                });
                ramps += 1;
            } else {
                break;
            }
        }
        assert_eq!((blocks, ramps), (180, 40));
    }
    /// Highest support close enough to the feet, including sloped surfaces.
    pub fn ground_surface(&self, p: Vec3) -> Option<(f32, Vec3)> {
        let blocks = self
            .collision_solids()
            .filter(|s| {
                p.x >= s.min.x
                    && p.x <= s.max.x
                    && p.z >= s.min.z
                    && p.z <= s.max.z
                    && s.max.y <= p.y + 0.28
            })
            .map(|s| (s.max.y, Vec3::Y));
        let slopes = self
            .continuous_surface(p.x, p.z)
            .filter(|s| s.0 <= p.y + 0.28);
        blocks.chain(slopes).max_by(|a, b| a.0.total_cmp(&b.0))
    }
    pub fn continuous_surface(&self, x: f32, z: f32) -> Option<(f32, Vec3)> {
        self.ramps
            .iter()
            .filter_map(|r| r.height(x, z).map(|h| (h, r.normal())))
            .chain(self.terrain.as_ref().and_then(|t| t.sample(x, z)))
            .max_by(|a, b| a.0.total_cmp(&b.0))
    }
    pub fn collision_solids(&self) -> impl Iterator<Item = &Solid> {
        self.solids
            .iter()
            .chain(self.platforms.iter().map(|p| &p.solid))
    }
    pub fn advance(&mut self, dt: f32) {
        self.time += dt as f64;
        for p in &mut self.platforms {
            p.advance(self.time, dt);
        }
    }
    pub fn reset_platforms(&mut self) {
        self.time = 0.;
        for p in &mut self.platforms {
            p.reset(0.);
        }
    }
    pub fn floor_height(&self, p: Vec3) -> Option<f32> {
        let blocks = self
            .collision_solids()
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
        let terrain = self
            .terrain
            .as_ref()
            .and_then(|t| t.sample(p.x, p.z))
            .map(|s| s.0)
            .filter(|h| *h <= p.y + 0.15);
        blocks.chain(ramps).chain(terrain).max_by(f32::total_cmp)
    }
}
