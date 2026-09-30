use super::{
    lighting::{detail_texture, shadow_matrix, DETAIL_SIZE, SHADOW_SIZE, SUN},
    mesh::{Mesh, Vertex},
};
use glam::{Mat4, Vec3};
use miniquad::*;

pub const SKY: [f32; 3] = [0.68, 0.80, 0.85];
const CAPACITY: usize = 60000;
pub struct GpuMesh {
    bindings: Bindings,
    shadow_bindings: Bindings,
    count: i32,
    shadow_count: i32,
    min: Vec3,
    max: Vec3,
}
pub struct Renderer {
    pub ctx: Box<dyn RenderingBackend>,
    world_pipeline: Pipeline,
    ui_pipeline: Pipeline,
    shadow_pipeline: Pipeline,
    sky_pipeline: Pipeline,
    sky_bindings: Bindings,
    stream: GpuMesh,
    target: RenderPass,
    shadow_target: RenderPass,
    light_matrix: Mat4,
    detail: TextureId,
    shadow: TextureId,
    pub size: (u32, u32),
    present_pipeline: Pipeline,
    present_bindings: Bindings,
}
#[repr(C)]
struct WorldUniforms {
    matrix: [f32; 16],
    light_matrix: [f32; 16],
    eye: [f32; 3],
    sun: [f32; 3],
}
#[repr(C)]
struct MatrixUniforms {
    matrix: [f32; 16],
}
#[repr(C)]
struct SkyUniforms {
    inverse_matrix: [f32; 16],
    eye: [f32; 3],
}
impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
impl Renderer {
    pub fn new() -> Self {
        let mut ctx = window::new_rendering_backend();
        let compile = |ctx: &mut dyn RenderingBackend,
                       vertex,
                       fragment,
                       images: Vec<String>,
                       uniforms: Vec<UniformDesc>| {
            ctx.new_shader(
                ShaderSource::Glsl { vertex, fragment },
                ShaderMeta {
                    images,
                    uniforms: UniformBlockLayout { uniforms },
                },
            )
            .expect("graphics shader")
        };
        let world_shader = compile(
            ctx.as_mut(),
            include_str!("shaders/world.vert"),
            include_str!("shaders/world.frag"),
            vec!["detail".into(), "shadow_map".into()],
            vec![
                UniformDesc::new("matrix", UniformType::Mat4),
                UniformDesc::new("light_matrix", UniformType::Mat4),
                UniformDesc::new("eye", UniformType::Float3),
                UniformDesc::new("sun", UniformType::Float3),
            ],
        );
        let ui_shader = compile(
            ctx.as_mut(),
            include_str!("shaders/ui.vert"),
            include_str!("shaders/ui.frag"),
            vec![],
            vec![UniformDesc::new("matrix", UniformType::Mat4)],
        );
        let shadow_shader = compile(
            ctx.as_mut(),
            include_str!("shaders/shadow.vert"),
            include_str!("shaders/shadow.frag"),
            vec![],
            vec![UniformDesc::new("matrix", UniformType::Mat4)],
        );
        let sky_shader = compile(
            ctx.as_mut(),
            include_str!("shaders/sky.vert"),
            include_str!("shaders/sky.frag"),
            vec![],
            vec![
                UniformDesc::new("inverse_matrix", UniformType::Mat4),
                UniformDesc::new("eye", UniformType::Float3),
            ],
        );
        let attrs = [
            VertexAttribute::new("in_pos", VertexFormat::Float3),
            VertexAttribute::new("in_normal", VertexFormat::Float3),
            VertexAttribute::new("in_color", VertexFormat::Float3),
            VertexAttribute::new("in_uv", VertexFormat::Float2),
            VertexAttribute::new("in_tangent", VertexFormat::Float3),
            VertexAttribute::new("in_style", VertexFormat::Float1),
        ];
        let layouts = [BufferLayout {
            stride: std::mem::size_of::<Vertex>() as i32,
            ..Default::default()
        }];
        let world_pipeline = ctx.new_pipeline(
            &layouts,
            &attrs,
            world_shader,
            PipelineParams {
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                ..Default::default()
            },
        );
        let ui_pipeline = ctx.new_pipeline(&layouts, &attrs, ui_shader, PipelineParams::default());
        let shadow_pipeline = ctx.new_pipeline(
            &layouts,
            &[VertexAttribute::new("in_pos", VertexFormat::Float3)],
            shadow_shader,
            PipelineParams {
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                ..Default::default()
            },
        );
        let pixels = detail_texture();
        let detail = ctx.new_texture_from_data_and_format(
            &pixels,
            TextureParams {
                width: DETAIL_SIZE as u32,
                height: DETAIL_SIZE as u32,
                wrap: TextureWrap::Repeat,
                min_filter: FilterMode::Linear,
                mag_filter: FilterMode::Linear,
                mipmap_filter: MipmapFilterMode::Linear,
                allocate_mipmaps: true,
                ..Default::default()
            },
        );
        ctx.texture_generate_mipmaps(detail);
        let shadow = ctx.new_render_texture(TextureParams {
            width: SHADOW_SIZE,
            height: SHADOW_SIZE,
            format: TextureFormat::RGBA8,
            min_filter: FilterMode::Nearest,
            mag_filter: FilterMode::Nearest,
            ..Default::default()
        });
        let shadow_depth = ctx.new_render_texture(TextureParams {
            width: SHADOW_SIZE,
            height: SHADOW_SIZE,
            format: TextureFormat::Depth,
            ..Default::default()
        });
        let shadow_target = ctx.new_render_pass(shadow, Some(shadow_depth));
        let stream_bindings = Bindings {
            vertex_buffers: vec![ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Stream,
                BufferSource::empty::<Vertex>(CAPACITY),
            )],
            index_buffer: ctx.new_buffer(
                BufferType::IndexBuffer,
                BufferUsage::Stream,
                BufferSource::empty::<u16>(CAPACITY * 2),
            ),
            images: vec![detail, shadow],
        };
        let mut stream_shadow = stream_bindings.clone();
        stream_shadow.images.clear();
        let stream = GpuMesh {
            bindings: stream_bindings,
            shadow_bindings: stream_shadow,
            count: 0,
            shadow_count: 0,
            min: Vec3::ZERO,
            max: Vec3::ZERO,
        };
        let size = (1280, 720);
        let (target, color) = make_target(ctx.as_mut(), size);
        let present_shader = compile(
            ctx.as_mut(),
            include_str!("shaders/present.vert"),
            include_str!("shaders/present.frag"),
            vec!["image".into()],
            vec![],
        );
        let screen_attrs = [VertexAttribute::new("pos", VertexFormat::Float2)];
        let present_pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &screen_attrs,
            present_shader,
            PipelineParams::default(),
        );
        let sky_pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &screen_attrs,
            sky_shader,
            PipelineParams::default(),
        );
        let vertices: [(f32, f32); 4] = [(-1., -1.), (1., -1.), (1., 1.), (-1., 1.)];
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let present_bindings = Bindings {
            vertex_buffers: vec![ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&vertices),
            )],
            index_buffer: ctx.new_buffer(
                BufferType::IndexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&indices),
            ),
            images: vec![color],
        };
        let mut sky_bindings = present_bindings.clone();
        sky_bindings.images.clear();
        Self {
            ctx,
            world_pipeline,
            ui_pipeline,
            shadow_pipeline,
            sky_pipeline,
            sky_bindings,
            stream,
            target,
            shadow_target,
            light_matrix: Mat4::IDENTITY,
            detail,
            shadow,
            size,
            present_pipeline,
            present_bindings,
        }
    }
    pub fn upload(&mut self, mesh: &Mesh) -> GpuMesh {
        assert!(
            mesh.vertices.len() < CAPACITY,
            "static scene exceeds u16 mesh budget"
        );
        let bindings = Bindings {
            vertex_buffers: vec![self.ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&mesh.vertices),
            )],
            index_buffer: self.ctx.new_buffer(
                BufferType::IndexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&mesh.indices),
            ),
            images: vec![self.detail, self.shadow],
        };
        let indices = shadow_indices(mesh);
        let shadow_bindings = Bindings {
            vertex_buffers: bindings.vertex_buffers.clone(),
            index_buffer: self.ctx.new_buffer(
                BufferType::IndexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&indices),
            ),
            images: vec![],
        };
        GpuMesh {
            bindings,
            shadow_bindings,
            count: mesh.indices.len() as i32,
            shadow_count: indices.len() as i32,
            min: mesh
                .vertices
                .iter()
                .fold(Vec3::splat(f32::INFINITY), |a, v| {
                    a.min(Vec3::from_array(v.pos))
                }),
            max: mesh
                .vertices
                .iter()
                .fold(Vec3::splat(f32::NEG_INFINITY), |a, v| {
                    a.max(Vec3::from_array(v.pos))
                }),
        }
    }
    fn stream_vertices(&mut self, mesh: &Mesh) {
        assert!(mesh.vertices.len() < CAPACITY && mesh.indices.len() < CAPACITY * 2);
        self.ctx.buffer_update(
            self.stream.bindings.vertex_buffers[0],
            BufferSource::slice(&mesh.vertices),
        );
    }
    /// Both passes use the same interpolated moving platforms/character pose.
    pub fn shadows(&mut self, scene: &[GpuMesh], dynamic: &Mesh, focus: Vec3) {
        self.light_matrix = shadow_matrix(focus);
        self.ctx.begin_pass(
            Some(self.shadow_target),
            PassAction::clear_color(1., 1., 1., 1.),
        );
        self.ctx.apply_pipeline(&self.shadow_pipeline);
        self.ctx
            .apply_uniforms(UniformsSource::table(&MatrixUniforms {
                matrix: self.light_matrix.to_cols_array(),
            }));
        for mesh in scene {
            let mut min = Vec3::splat(f32::INFINITY);
            let mut max = Vec3::splat(f32::NEG_INFINITY);
            for x in [mesh.min.x, mesh.max.x] {
                for y in [mesh.min.y, mesh.max.y] {
                    for z in [mesh.min.z, mesh.max.z] {
                        let p = self.light_matrix.transform_point3(Vec3::new(x, y, z));
                        min = min.min(p);
                        max = max.max(p);
                    }
                }
            }
            if min.cmple(Vec3::ONE).all() && max.cmpge(Vec3::NEG_ONE).all() {
                self.ctx.apply_bindings(&mesh.shadow_bindings);
                self.ctx.draw(0, mesh.shadow_count, 1);
            }
        }
        self.stream_vertices(dynamic);
        let indices = shadow_indices(dynamic);
        self.ctx.buffer_update(
            self.stream.bindings.index_buffer,
            BufferSource::slice(&indices),
        );
        self.ctx.apply_bindings(&self.stream.shadow_bindings);
        self.ctx.draw(0, indices.len() as i32, 1);
        self.ctx.end_render_pass();
    }
    pub fn begin(&mut self, matrix: Mat4, eye: Vec3) {
        self.ctx.begin_pass(
            Some(self.target),
            PassAction::clear_color(SKY[0], SKY[1], SKY[2], 1.),
        );
        self.ctx.apply_pipeline(&self.sky_pipeline);
        self.ctx.apply_bindings(&self.sky_bindings);
        self.ctx.apply_uniforms(UniformsSource::table(&SkyUniforms {
            inverse_matrix: matrix.inverse().to_cols_array(),
            eye: eye.to_array(),
        }));
        self.ctx.draw(0, 6, 1);
    }
    pub fn resize(&mut self, size: (u32, u32)) {
        if self.size == size {
            return;
        }
        let (target, color) = make_target(self.ctx.as_mut(), size);
        self.ctx.delete_render_pass(self.target);
        self.target = target;
        self.size = size;
        self.present_bindings.images[0] = color;
    }
    fn world_uniforms(&self, matrix: Mat4, eye: Vec3) -> WorldUniforms {
        WorldUniforms {
            matrix: matrix.to_cols_array(),
            light_matrix: self.light_matrix.to_cols_array(),
            eye: eye.to_array(),
            sun: SUN.normalize().to_array(),
        }
    }
    pub fn static_mesh(&mut self, mesh: &GpuMesh, matrix: Mat4, eye: Vec3) {
        self.ctx.apply_pipeline(&self.world_pipeline);
        self.ctx.apply_bindings(&mesh.bindings);
        let uniforms = self.world_uniforms(matrix, eye);
        self.ctx.apply_uniforms(UniformsSource::table(&uniforms));
        self.ctx.draw(0, mesh.count, 1);
    }
    pub fn dynamic(&mut self, mesh: &Mesh, matrix: Mat4, eye: Vec3, ui: bool) {
        if mesh.indices.is_empty() {
            return;
        }
        self.stream_vertices(mesh);
        self.ctx.buffer_update(
            self.stream.bindings.index_buffer,
            BufferSource::slice(&mesh.indices),
        );
        if ui {
            self.ctx.apply_pipeline(&self.ui_pipeline);
            self.ctx.apply_bindings(&self.stream.shadow_bindings);
            self.ctx
                .apply_uniforms(UniformsSource::table(&MatrixUniforms {
                    matrix: matrix.to_cols_array(),
                }));
        } else {
            self.ctx.apply_pipeline(&self.world_pipeline);
            self.ctx.apply_bindings(&self.stream.bindings);
            let uniforms = self.world_uniforms(matrix, eye);
            self.ctx.apply_uniforms(UniformsSource::table(&uniforms));
        }
        self.ctx.draw(0, mesh.indices.len() as i32, 1);
    }
    pub fn finish(&mut self, screenshot: Option<&str>) {
        self.ctx.end_render_pass();
        if let Some(path) = screenshot {
            self.capture(path).expect("save screenshot");
        }
        self.ctx
            .begin_default_pass(PassAction::clear_color(0.025, 0.035, 0.045, 1.));
        let (w, h) = window::screen_size();
        let (x, y, vw, vh) = viewport(w, h, self.size);
        self.ctx
            .apply_viewport(x as i32, y as i32, vw as i32, vh as i32);
        self.ctx.apply_pipeline(&self.present_pipeline);
        self.ctx.apply_bindings(&self.present_bindings);
        self.ctx.draw(0, 6, 1);
        self.ctx.end_render_pass();
        self.ctx.commit_frame();
    }
    fn capture(&mut self, path: &str) -> std::io::Result<()> {
        use std::io::Write;
        let (width, height) = (self.size.0 as usize, self.size.1 as usize);
        let mut pixels = vec![0u8; width * height * 4];
        self.ctx
            .texture_read_pixels(self.present_bindings.images[0], &mut pixels);
        let mut file = std::fs::File::create(path)?;
        write!(file, "P6\n{} {}\n255\n", width, height)?;
        for y in (0..height).rev() {
            for x in 0..width {
                let i = (y * width + x) * 4;
                file.write_all(&pixels[i..i + 3])?;
            }
        }
        Ok(())
    }
}
/// Markings, beacons and dust do not cast distracting tiny shadows.
pub fn shadow_indices(mesh: &Mesh) -> Vec<u16> {
    mesh.indices
        .as_chunks::<3>()
        .0
        .iter()
        .filter(|tri| mesh.vertices[tri[0] as usize].style != 2.)
        .flatten()
        .copied()
        .collect()
}

/// Centre a render resolution in a window, preserving aspect with letterboxing.
pub fn viewport(w: f32, h: f32, size: (u32, u32)) -> (f32, f32, f32, f32) {
    let scale = (w / size.0 as f32).min(h / size.1 as f32);
    let (vw, vh) = (size.0 as f32 * scale, size.1 as f32 * scale);
    ((w - vw) * 0.5, (h - vh) * 0.5, vw, vh)
}
fn make_target(ctx: &mut dyn RenderingBackend, size: (u32, u32)) -> (RenderPass, TextureId) {
    let color = ctx.new_render_texture(TextureParams {
        width: size.0,
        height: size.1,
        format: TextureFormat::RGBA8,
        min_filter: FilterMode::Linear,
        mag_filter: FilterMode::Linear,
        ..Default::default()
    });
    let depth = ctx.new_render_texture(TextureParams {
        width: size.0,
        height: size.1,
        // Thin surface caps and the courtyard skirt need more precision over
        // the expanded view range than a 16-bit depth buffer can provide.
        format: TextureFormat::Depth32,
        sample_count: if ctx.info().features.resolve_attachments {
            4
        } else {
            1
        },
        ..Default::default()
    });
    if ctx.info().features.resolve_attachments {
        let multisample = ctx.new_render_texture(TextureParams {
            width: size.0,
            height: size.1,
            format: TextureFormat::RGBA8,
            sample_count: 4,
            ..Default::default()
        });
        (
            ctx.new_render_pass_mrt(&[multisample], Some(&[color]), Some(depth)),
            color,
        )
    } else {
        (ctx.new_render_pass(color, Some(depth)), color)
    }
}
