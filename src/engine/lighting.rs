//! Shared sun direction, stable shadow camera and embedded material texture.
use glam::{vec3, Mat4, Vec3};

pub const SHADOW_SIZE: u32 = 2048;
pub const SHADOW_RADIUS: f32 = 52.;
pub const SUN: Vec3 = Vec3::new(-0.48, 0.82, 0.31);
pub const DETAIL_SIZE: usize = 256;

/// Snap the shadow projection in light space to prevent sub-texel swimming.
pub fn shadow_matrix(focus: Vec3) -> Mat4 {
    let light = SUN.normalize();
    let basis = Mat4::look_at_rh(light * 85., Vec3::ZERO, Vec3::Y);
    let p = basis.transform_point3(focus);
    let texel = SHADOW_RADIUS * 2. / SHADOW_SIZE as f32;
    let snapped = vec3(
        (p.x / texel).round() * texel,
        (p.y / texel).round() * texel,
        p.z,
    );
    let center = basis.inverse().transform_point3(snapped);
    Mat4::orthographic_rh_gl(
        -SHADOW_RADIUS,
        SHADOW_RADIUS,
        -SHADOW_RADIUS,
        SHADOW_RADIUS,
        0.1,
        185.,
    ) * Mat4::look_at_rh(center + light * 85., center, Vec3::Y)
}

fn hash(x: usize, y: usize) -> f32 {
    let mut n = (x as u32).wrapping_mul(374761393) ^ (y as u32).wrapping_mul(668265263);
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    (n ^ (n >> 16)) as f32 / u32::MAX as f32
}
fn noise(x: f32, y: f32, period: usize) -> f32 {
    let (i, j) = (x.floor() as usize, y.floor() as usize);
    let (u, v) = (x.fract(), y.fract());
    let (u, v) = (u * u * (3. - 2. * u), v * v * (3. - 2. * v));
    let a = hash(i % period, j % period);
    let b = hash((i + 1) % period, j % period);
    let c = hash(i % period, (j + 1) % period);
    let d = hash((i + 1) % period, (j + 1) % period);
    (a + (b - a) * u) * (1. - v) + (c + (d - c) * u) * v
}
/// RGBA: albedo variation, two normal slopes, roughness variation. Periodic
/// noise and wrapped differences make every repeat seamless. Mipmaps remove
/// fine detail in the distance instead of producing crawling/shimmering pixels.
pub fn detail_texture() -> Vec<u8> {
    let mut height = vec![0.; DETAIL_SIZE * DETAIL_SIZE];
    for y in 0..DETAIL_SIZE {
        for x in 0..DETAIL_SIZE {
            let (u, v) = (x as f32 / DETAIL_SIZE as f32, y as f32 / DETAIL_SIZE as f32);
            height[y * DETAIL_SIZE + x] = noise(u * 4., v * 4., 4) * 0.55
                + noise(u * 16., v * 16., 16) * 0.25
                + noise(u * 64., v * 64., 64) * 0.20;
        }
    }
    let byte = |v: f32| (v.clamp(0., 1.) * 255.).round() as u8;
    let mut pixels = Vec::with_capacity(DETAIL_SIZE * DETAIL_SIZE * 4);
    for y in 0..DETAIL_SIZE {
        for x in 0..DETAIL_SIZE {
            let h = height[y * DETAIL_SIZE + x];
            let dx = height[y * DETAIL_SIZE + (x + 1) % DETAIL_SIZE]
                - height[y * DETAIL_SIZE + (x + DETAIL_SIZE - 1) % DETAIL_SIZE];
            let dy = height[((y + 1) % DETAIL_SIZE) * DETAIL_SIZE + x]
                - height[((y + DETAIL_SIZE - 1) % DETAIL_SIZE) * DETAIL_SIZE + x];
            pixels.extend([
                byte(0.5 + (h - 0.5) * 0.65),
                byte(0.5 - dx * 1.8),
                byte(0.5 - dy * 1.8),
                byte(0.5 + (h - 0.5) * 0.5),
            ]);
        }
    }
    pixels
}
