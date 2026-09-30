use glam::{vec3, Mat4, Vec3};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
    pub uv: [f32; 2],
    pub tangent: [f32; 3],
    /// 0 = concrete, 1 = pavers, 2 = markings, 3 = paint, 4 = rubber, 5 = terrain.
    pub style: f32,
}

#[derive(Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Mesh {
    pub fn triangle(&mut self, a: Vec3, b: Vec3, c: Vec3, color: [f32; 3], style: f32) {
        let n = (b - a).cross(c - a).normalize_or_zero();
        let tangent = (b - a).normalize_or_zero();
        let bitangent = n.cross(tangent);
        let start = self.vertices.len() as u16;
        for p in [a, b, c] {
            self.vertices.push(Vertex {
                pos: p.to_array(),
                normal: n.to_array(),
                uv: [(p - a).dot(tangent), (p - a).dot(bitangent)],
                tangent: tangent.to_array(),
                color,
                style,
            });
        }
        self.indices.extend([start, start + 1, start + 2]);
    }
    pub fn quad(&mut self, p: [Vec3; 4], color: [f32; 3], style: f32) {
        let normal = (p[1] - p[0])
            .cross(p[2] - p[0])
            .normalize_or_zero()
            .to_array();
        let tangent = (p[1] - p[0]).normalize_or_zero();
        let bitangent = Vec3::from_array(normal).cross(tangent);
        let start = self.vertices.len() as u16;
        for pos in p {
            self.vertices.push(Vertex {
                pos: pos.to_array(),
                normal,
                uv: [(pos - p[0]).dot(tangent), (pos - p[0]).dot(bitangent)],
                tangent: tangent.to_array(),
                color,
                style,
            });
        }
        self.indices
            .extend([start, start + 1, start + 2, start, start + 2, start + 3]);
    }
    pub fn cube(&mut self, center: Vec3, size: Vec3, color: [f32; 3], style: f32) {
        if style != 2. && style != 1. && size.min_element() > 0.12 && size.max_element() < 80. {
            self.rounded_box(
                center,
                size,
                size.min_element().mul_add(0.16, 0.).min(0.12),
                color,
                style,
            );
            return;
        }
        let a = center - size * 0.5;
        let b = center + size * 0.5;
        self.quad(
            [
                vec3(a.x, b.y, a.z),
                vec3(a.x, b.y, b.z),
                vec3(b.x, b.y, b.z),
                vec3(b.x, b.y, a.z),
            ],
            color,
            style,
        );
        self.quad(
            [
                vec3(a.x, a.y, b.z),
                vec3(a.x, a.y, a.z),
                vec3(b.x, a.y, a.z),
                vec3(b.x, a.y, b.z),
            ],
            color,
            style,
        );
        self.quad(
            [
                vec3(a.x, a.y, a.z),
                vec3(a.x, a.y, b.z),
                vec3(a.x, b.y, b.z),
                vec3(a.x, b.y, a.z),
            ],
            color,
            style,
        );
        self.quad(
            [
                vec3(b.x, a.y, b.z),
                vec3(b.x, a.y, a.z),
                vec3(b.x, b.y, a.z),
                vec3(b.x, b.y, b.z),
            ],
            color,
            style,
        );
        self.quad(
            [
                vec3(b.x, a.y, a.z),
                vec3(a.x, a.y, a.z),
                vec3(a.x, b.y, a.z),
                vec3(b.x, b.y, a.z),
            ],
            color,
            style,
        );
        self.quad(
            [
                vec3(a.x, a.y, b.z),
                vec3(b.x, a.y, b.z),
                vec3(b.x, b.y, b.z),
                vec3(a.x, b.y, b.z),
            ],
            color,
            style,
        );
    }
    /// A two-segment rounded bevel with smooth analytic normals. Flat centres
    /// preserve the authored dimensions and all walking/landing surfaces.
    pub fn rounded_box(
        &mut self,
        center: Vec3,
        size: Vec3,
        radius: f32,
        color: [f32; 3],
        style: f32,
    ) {
        let half = size * 0.5;
        let r = radius.clamp(0.001, half.min_element() * 0.95);
        let inner = half - Vec3::splat(r);
        for (normal, u, v) in [
            (Vec3::Y, Vec3::Z, Vec3::X),
            (Vec3::NEG_Y, Vec3::NEG_Z, Vec3::X),
            (Vec3::X, Vec3::NEG_Z, Vec3::Y),
            (Vec3::NEG_X, Vec3::Z, Vec3::Y),
            (Vec3::Z, Vec3::X, Vec3::Y),
            (Vec3::NEG_Z, Vec3::NEG_X, Vec3::Y),
        ] {
            let hu = half.dot(u.abs());
            let hv = half.dot(v.abs());
            let us = [-hu, -hu + r, hu - r, hu];
            let vs = [-hv, -hv + r, hv - r, hv];
            for i in 0..3 {
                for j in 0..3 {
                    let start = self.vertices.len() as u16;
                    for (x, y) in [
                        (us[i], vs[j]),
                        (us[i + 1], vs[j]),
                        (us[i + 1], vs[j + 1]),
                        (us[i], vs[j + 1]),
                    ] {
                        let p = normal * half.dot(normal.abs()) + u * x + v * y;
                        let q = p.clamp(-inner, inner);
                        let n = (p - q).normalize_or_zero();
                        self.vertices.push(Vertex {
                            pos: (center + q + n * r).to_array(),
                            normal: n.to_array(),
                            color,
                            style,
                            uv: [x, y],
                            tangent: u.to_array(),
                        });
                    }
                    self.indices
                        .extend([start, start + 1, start + 2, start, start + 2, start + 3]);
                }
            }
        }
    }
    pub fn transform_from(&mut self, start: usize, matrix: Mat4) {
        for v in &mut self.vertices[start..] {
            v.pos = matrix.transform_point3(Vec3::from_array(v.pos)).to_array();
            v.tangent = matrix
                .transform_vector3(Vec3::from_array(v.tangent))
                .normalize_or_zero()
                .to_array();
            v.normal = matrix
                .transform_vector3(Vec3::from_array(v.normal))
                .normalize_or_zero()
                .to_array();
        }
    }
    pub fn disc(&mut self, center: Vec3, radius: f32, color: [f32; 3]) {
        for i in 0..32 {
            let a = i as f32 * std::f32::consts::TAU / 32.;
            let b = (i + 1) as f32 * std::f32::consts::TAU / 32.;
            self.triangle(
                center,
                center + vec3(a.sin(), 0., a.cos()) * radius,
                center + vec3(b.sin(), 0., b.cos()) * radius,
                color,
                2.,
            );
        }
    }
    pub fn ring(&mut self, center: Vec3, radius: f32, thickness: f32, color: [f32; 3]) {
        for i in 0..48 {
            let a = i as f32 * std::f32::consts::TAU / 48.;
            let b = (i + 1) as f32 * std::f32::consts::TAU / 48.;
            let av = vec3(a.sin(), 0., a.cos());
            let bv = vec3(b.sin(), 0., b.cos());
            self.quad(
                [
                    center + av * radius,
                    center + bv * radius,
                    center + bv * (radius + thickness),
                    center + av * (radius + thickness),
                ],
                color,
                2.,
            );
        }
    }
}
