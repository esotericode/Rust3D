use glam::{vec3, Mat4, Vec3};
use stride::engine::{
    lighting::{detail_texture, shadow_matrix, DETAIL_SIZE, SHADOW_RADIUS, SHADOW_SIZE},
    mesh::Mesh,
    renderer::shadow_indices,
};

#[test]
fn rounded_geometry_preserves_bounds_and_has_smooth_unit_normals() {
    let mut mesh = Mesh::default();
    let center = vec3(4., 3., -5.);
    let half = vec3(2., 1., 3.);
    mesh.rounded_box(center, half * 2., 0.12, [0.6, 0.7, 0.7], 0.);
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    let mut diagonal = false;
    for v in &mesh.vertices {
        let p = Vec3::from_array(v.pos);
        let n = Vec3::from_array(v.normal);
        assert!(p.is_finite() && (n.length() - 1.).abs() < 0.0001);
        assert!(
            p.cmpge(center - half - Vec3::splat(0.0001)).all()
                && p.cmple(center + half + Vec3::splat(0.0001)).all()
        );
        min = min.min(p);
        max = max.max(p);
        diagonal |= n.abs().max_element() < 0.9;
    }
    assert!(
        (min - (center - half)).length() < 0.0001
            && (max - (center + half)).length() < 0.0001
            && diagonal
    );
    for t in mesh.indices.as_chunks::<3>().0 {
        let (a, b, c) = (
            &mesh.vertices[t[0] as usize],
            &mesh.vertices[t[1] as usize],
            &mesh.vertices[t[2] as usize],
        );
        let face = (Vec3::from_array(b.pos) - Vec3::from_array(a.pos))
            .cross(Vec3::from_array(c.pos) - Vec3::from_array(a.pos));
        assert!(face.dot(Vec3::from_array(a.normal)) > 0.);
    }
}
#[test]
fn material_coordinates_follow_objects_without_swimming() {
    let mut a = Mesh::default();
    let mut b = Mesh::default();
    a.cube(Vec3::ZERO, Vec3::splat(2.), [1.; 3], 3.);
    b.cube(vec3(40., 8., -12.), Vec3::splat(2.), [1.; 3], 3.);
    assert_eq!(
        a.vertices.iter().map(|v| v.uv).collect::<Vec<_>>(),
        b.vertices.iter().map(|v| v.uv).collect::<Vec<_>>()
    );
    let coords = a.vertices.iter().map(|v| v.uv).collect::<Vec<_>>();
    a.transform_from(0, Mat4::from_rotation_y(0.8));
    assert_eq!(coords, a.vertices.iter().map(|v| v.uv).collect::<Vec<_>>());
    assert!(a
        .vertices
        .iter()
        .all(|v| (Vec3::from_array(v.tangent).length() - 1.).abs() < 0.001));
}
#[test]
fn embedded_texture_is_deterministic_and_has_surface_detail() {
    let a = detail_texture();
    assert_eq!(a.len(), DETAIL_SIZE * DETAIL_SIZE * 4);
    assert_eq!(a, detail_texture());
    for channel in 0..4 {
        let values = a
            .iter()
            .skip(channel)
            .step_by(4)
            .copied()
            .collect::<Vec<_>>();
        assert!(values.iter().max().unwrap() - values.iter().min().unwrap() > 12);
    }
    // Wrapped edges have the same scale of variation as internal neighbours.
    let seam = (0..DETAIL_SIZE)
        .map(|y| {
            (a[(y * DETAIL_SIZE) * 4] as i32 - a[(y * DETAIL_SIZE + DETAIL_SIZE - 1) * 4] as i32)
                .abs()
        })
        .sum::<i32>();
    assert!(seam < DETAIL_SIZE as i32 * 15);
}
#[test]
fn shadow_projection_covers_local_course_and_excludes_markings() {
    let focus = vec3(153., 41., 93.);
    let matrix = shadow_matrix(focus);
    for p in [focus, focus - Vec3::Y * 41., focus + Vec3::Y * 10.] {
        let q = matrix * p.extend(1.);
        assert!(q.truncate().abs().max_element() < 1.);
    }
    let texel = SHADOW_RADIUS * 2. / SHADOW_SIZE as f32;
    let m2 = shadow_matrix(focus + Vec3::X * texel * 0.1);
    assert!((matrix.w_axis - m2.w_axis).truncate().length() < 0.001);
    let mut mesh = Mesh::default();
    mesh.cube(Vec3::ZERO, Vec3::ONE, [1.; 3], 0.);
    let count = mesh.indices.len();
    mesh.cube(Vec3::Y * 2., Vec3::ONE, [1.; 3], 2.);
    assert_eq!(shadow_indices(&mesh).len(), count);
}
