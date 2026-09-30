//! Deterministic heightfield: rendering and collision share the same triangles.
use super::mesh::{Mesh, Vertex};
use glam::{vec2, vec3, Vec2, Vec3};

pub const MAP_SIZE: Vec2 = Vec2::new(1440., 1680.);
pub const MAP_MIN: Vec2 = Vec2::new(-620., -865.);
pub const CELL: f32 = 6.;
pub const SEED: u32 = 0x51de_2026;

pub struct Region {
    pub name: &'static str,
    pub point: Vec2,
    pub hint: &'static str,
}
pub const REGIONS: [Region; 5] = [
    Region {
        name: "LONG RUN",
        point: Vec2::new(100., 740.),
        hint: "OPEN LANE / CHAIN LONG JUMP, DIVE, ROLLOUT",
    },
    Region {
        name: "ROLLING BASIN",
        point: Vec2::new(-350., 350.),
        hint: "WIDE HILLS / CARRY SPEED THROUGH THE VALLEYS",
    },
    Region {
        name: "RIDGE DESCENT",
        point: Vec2::new(550., -570.),
        hint: "DOWNHILL ADDS SPEED / STEER EARLY AND BRAKE",
    },
    Region {
        name: "MOUNTAIN PASS",
        point: Vec2::new(-420., -510.),
        hint: "STEEP FACES SLIDE / FIND THE GENTLER APPROACH",
    },
    Region {
        name: "BLOCK FIELDS",
        point: Vec2::new(470., 380.),
        hint: "VARIED BLOCKS AND RAMPS / KEEP YOUR LINE",
    },
];

#[derive(Clone, Debug)]
pub struct Terrain {
    pub min: Vec2,
    pub cols: usize,
    pub rows: usize,
    pub cell: f32,
    pub heights: Vec<f32>,
}
impl Terrain {
    pub fn from_fn(
        min: Vec2,
        cols: usize,
        rows: usize,
        cell: f32,
        f: impl Fn(f32, f32) -> f32,
    ) -> Self {
        assert!(cols > 0 && rows > 0 && cell > 0.);
        let mut heights = Vec::with_capacity((cols + 1) * (rows + 1));
        for z in 0..=rows {
            for x in 0..=cols {
                heights.push(f(min.x + x as f32 * cell, min.y + z as f32 * cell));
            }
        }
        Self {
            min,
            cols,
            rows,
            cell,
            heights,
        }
    }
    pub fn highlands() -> Self {
        Self::from_fn(MAP_MIN, 240, 280, CELL, landscape_height)
    }
    fn vertex_height(&self, x: usize, z: usize) -> f32 {
        self.heights[z * (self.cols + 1) + x]
    }
    pub fn sample(&self, x: f32, z: f32) -> Option<(f32, Vec3)> {
        let p = (vec2(x, z) - self.min) / self.cell;
        if p.x < 0. || p.y < 0. || p.x > self.cols as f32 || p.y > self.rows as f32 {
            return None;
        }
        let (i, j) = (
            (p.x.floor() as usize).min(self.cols - 1),
            (p.y.floor() as usize).min(self.rows - 1),
        );
        let (u, v) = (p.x - i as f32, p.y - j as f32);
        let (a, b, c, d) = (
            self.vertex_height(i, j),
            self.vertex_height(i + 1, j),
            self.vertex_height(i, j + 1),
            self.vertex_height(i + 1, j + 1),
        );
        let (height, dx, dz) = if u >= v {
            (a + (b - a) * u + (d - b) * v, b - a, d - b)
        } else {
            (a + (d - c) * u + (c - a) * v, d - c, c - a)
        };
        Some((
            height,
            vec3(-dx / self.cell, 1., -dz / self.cell).normalize(),
        ))
    }
    fn smooth_normal(&self, x: usize, z: usize) -> Vec3 {
        let (a, b, c, d) = (
            x.saturating_sub(1),
            (x + 1).min(self.cols),
            z.saturating_sub(1),
            (z + 1).min(self.rows),
        );
        vec3(
            -(self.vertex_height(b, z) - self.vertex_height(a, z)) / ((b - a) as f32 * self.cell),
            1.,
            -(self.vertex_height(x, d) - self.vertex_height(x, c)) / ((d - c) as f32 * self.cell),
        )
        .normalize()
    }
    pub fn meshes(&self) -> Vec<Mesh> {
        let mut chunks = Vec::new();
        for z0 in (0..self.rows).step_by(32) {
            for x0 in (0..self.cols).step_by(32) {
                let (nx, nz) = ((self.cols - x0).min(32), (self.rows - z0).min(32));
                let mut m = Mesh::default();
                for z in z0..=z0 + nz {
                    for x in x0..=x0 + nx {
                        let pos = vec3(
                            self.min.x + x as f32 * self.cell,
                            self.vertex_height(x, z),
                            self.min.y + z as f32 * self.cell,
                        );
                        let normal = self.smooth_normal(x, z);
                        let rock = ((0.86 - normal.y) * 4.)
                            .clamp(0., 1.)
                            .max(((pos.y - 125.) / 70.).clamp(0., 1.));
                        let grass = vec3(0.40, 0.53, 0.34);
                        let color = grass.lerp(vec3(0.55, 0.54, 0.49), rock);
                        m.vertices.push(Vertex {
                            pos: pos.to_array(),
                            normal: normal.to_array(),
                            color: color.to_array(),
                            uv: [pos.x, pos.z],
                            tangent: Vec3::X.to_array(),
                            style: 5.,
                        });
                    }
                }
                for z in 0..nz {
                    for x in 0..nx {
                        let a = (z * (nx + 1) + x) as u16;
                        let b = a + 1;
                        let c = a + (nx + 1) as u16;
                        let d = c + 1;
                        m.indices.extend([a, c, d, a, d, b]);
                    }
                }
                chunks.push(m);
            }
        }
        chunks
    }
}
fn smooth(v: f32) -> f32 {
    let v = v.clamp(0., 1.);
    v * v * (3. - 2. * v)
}
fn landscape_height(x: f32, z: f32) -> f32 {
    let mut h = 18.
        + 12. * (x / 103.).sin() * (z / 137.).cos()
        + 7. * ((x + z) / 71.).sin()
        + 4. * ((x - z) / 47.).cos();
    for (cx, cz, height, radius) in [
        (-425., -510., 185., 100.),
        (555., -570., 215., 115.),
        (-455., 545., 120., 125.),
        (625., 565., 155., 105.),
    ] {
        h += height * (-((x - cx).powi(2) + (z - cz).powi(2)) / (2. * radius * radius)).exp();
    }
    // A 550 m, gently rolling north/south lane has no random obstacles.
    let lane = (1. - smooth((x - 100.).abs() / 55.))
        * smooth((z - 200.) / 100.)
        * smooth((810. - z) / 45.);
    h = h * (1. - lane) + (6. + 3. * (z / 110.).sin()) * lane;
    let reserve = ((x - 100.).abs() - 180.)
        .max((z + 25.).abs() - 210.)
        .max(0.);
    let blend = smooth(reserve / 85.);
    // Keep terrain well below the courtyard's rendered skirt and floor. Close
    // overlapping surfaces produce distant depth stripes in the large overview.
    h.max(0.) * blend - 0.5 * (1. - blend)
}
pub struct Seeded(pub u32);
impl Seeded {
    pub fn unit(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.0 >> 8) as f32 / 16777216.
    }
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.unit()
    }
}
