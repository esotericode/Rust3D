use glam::{vec3, Mat4, Vec3};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
    /// 0 = lit, 1 = ground grid, 2 = emissive/unlit.
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
        let start = self.vertices.len() as u16;
        for p in [a, b, c] {
            self.vertices.push(Vertex {
                pos: p.to_array(),
                normal: n.to_array(),
                color,
                style,
            });
        }
        self.indices.extend([start, start + 1, start + 2]);
    }
    pub fn quad(&mut self, p: [Vec3; 4], color: [f32; 3], style: f32) {
        self.triangle(p[0], p[1], p[2], color, style);
        self.triangle(p[0], p[2], p[3], color, style);
    }
    pub fn cube(&mut self, center: Vec3, size: Vec3, color: [f32; 3], style: f32) {
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
    pub fn transform_from(&mut self, start: usize, matrix: Mat4) {
        for v in &mut self.vertices[start..] {
            v.pos = matrix.transform_point3(Vec3::from_array(v.pos)).to_array();
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
