use super::mesh::{Mesh, Vertex};
use glam::{Mat4, Vec3};
use miniquad::*;

pub const SKY: [f32; 3] = [0.68, 0.80, 0.85];
const CAPACITY: usize = 60000;

pub struct GpuMesh {
    bindings: Bindings,
    count: i32,
}
pub struct Renderer {
    pub ctx: Box<dyn RenderingBackend>,
    world_pipeline: Pipeline,
    ui_pipeline: Pipeline,
    stream: GpuMesh,
    target: RenderPass,
    pub size: (u32, u32),
    present_pipeline: Pipeline,
    present_bindings: Bindings,
}

#[repr(C)]
struct Uniforms {
    matrix: [f32; 16],
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
        let uniforms = || ShaderMeta {
            images: vec![],
            uniforms: UniformBlockLayout {
                uniforms: vec![
                    UniformDesc::new("matrix", UniformType::Mat4),
                    UniformDesc::new("eye", UniformType::Float3),
                ],
            },
        };
        let world_shader = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: WORLD_VERTEX,
                    fragment: WORLD_FRAGMENT,
                },
                uniforms(),
            )
            .expect("world shader");
        let ui_shader = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: UI_VERTEX,
                    fragment: UI_FRAGMENT,
                },
                uniforms(),
            )
            .expect("UI shader");
        let attrs = [
            VertexAttribute::new("in_pos", VertexFormat::Float3),
            VertexAttribute::new("in_normal", VertexFormat::Float3),
            VertexAttribute::new("in_color", VertexFormat::Float3),
            VertexAttribute::new("in_style", VertexFormat::Float1),
        ];
        let layouts = [BufferLayout::default()];
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
        let stream = GpuMesh {
            bindings: Bindings {
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
                images: vec![],
            },
            count: 0,
        };
        let size = (1280, 720);
        let (target, color) = make_target(ctx.as_mut(), size);
        let shader = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: PRESENT_VERTEX,
                    fragment: PRESENT_FRAGMENT,
                },
                ShaderMeta {
                    images: vec!["image".into()],
                    uniforms: UniformBlockLayout { uniforms: vec![] },
                },
            )
            .expect("presentation shader");
        let present_pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[VertexAttribute::new("pos", VertexFormat::Float2)],
            shader,
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
        Self {
            ctx,
            world_pipeline,
            ui_pipeline,
            stream,
            target,
            size,
            present_pipeline,
            present_bindings,
        }
    }
    pub fn upload(&mut self, mesh: &Mesh) -> GpuMesh {
        GpuMesh {
            bindings: Bindings {
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
                images: vec![],
            },
            count: mesh.indices.len() as i32,
        }
    }
    pub fn begin(&mut self) {
        self.ctx.begin_pass(
            Some(self.target),
            PassAction::clear_color(SKY[0], SKY[1], SKY[2], 1.),
        );
    }
    pub fn resize(&mut self, size: (u32, u32)) {
        if self.size == size {
            return;
        }
        let (target, color) = make_target(self.ctx.as_mut(), size);
        // Miniquad deletes a pass's owned colour/depth textures with the pass.
        self.ctx.delete_render_pass(self.target);
        self.target = target;
        self.size = size;
        self.present_bindings.images[0] = color;
    }
    pub fn static_mesh(&mut self, mesh: &GpuMesh, matrix: Mat4, eye: Vec3) {
        self.ctx.apply_pipeline(&self.world_pipeline);
        self.ctx.apply_bindings(&mesh.bindings);
        self.ctx.apply_uniforms(UniformsSource::table(&Uniforms {
            matrix: matrix.to_cols_array(),
            eye: eye.to_array(),
        }));
        self.ctx.draw(0, mesh.count, 1);
    }
    pub fn dynamic(&mut self, mesh: &Mesh, matrix: Mat4, eye: Vec3, ui: bool) {
        if mesh.indices.is_empty() {
            return;
        }
        assert!(mesh.vertices.len() < CAPACITY && mesh.indices.len() < CAPACITY * 2);
        self.ctx.buffer_update(
            self.stream.bindings.vertex_buffers[0],
            BufferSource::slice(&mesh.vertices),
        );
        self.ctx.buffer_update(
            self.stream.bindings.index_buffer,
            BufferSource::slice(&mesh.indices),
        );
        self.ctx.apply_pipeline(if ui {
            &self.ui_pipeline
        } else {
            &self.world_pipeline
        });
        self.ctx.apply_bindings(&self.stream.bindings);
        self.ctx.apply_uniforms(UniformsSource::table(&Uniforms {
            matrix: matrix.to_cols_array(),
            eye: eye.to_array(),
        }));
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

    /// Capture the resolved game render target, before scaling for the window.
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
        format: TextureFormat::Depth,
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
const PRESENT_VERTEX: &str = r#"#version 100
attribute vec2 pos;
varying vec2 uv;
void main(){gl_Position=vec4(pos,0.0,1.0);uv=pos*0.5+0.5;}"#;
const PRESENT_FRAGMENT: &str = r#"#version 100
precision mediump float;
uniform sampler2D image;
varying vec2 uv;
void main(){gl_FragColor=texture2D(image,uv);}"#;

const WORLD_VERTEX: &str = r#"#version 100
attribute vec3 in_pos;
attribute vec3 in_normal;
attribute vec3 in_color;
attribute float in_style;
uniform mat4 matrix;
uniform vec3 eye;
varying vec3 color;
varying vec3 position;
varying float fog;
varying float grid;
void main() {
    gl_Position = matrix * vec4(in_pos,1.0);
    float light = 0.60 + 0.40 * max(dot(in_normal,normalize(vec3(-0.4,0.85,0.3))),0.0);
    color = in_color * (in_style > 1.5 ? 1.0 : light);
    position = in_pos;
    grid = in_style > 0.5 && in_style < 1.5 && in_normal.y > 0.9 ? 1.0 : 0.0;
    fog = clamp((length(eye-in_pos)-28.0)/95.0,0.0,0.8);
}"#;
const WORLD_FRAGMENT: &str = r#"#version 100
precision mediump float;
varying vec3 color;
varying vec3 position;
varying float fog;
varying float grid;
void main() {
    vec2 cell = abs(fract(position.xz/2.0 + 0.5)-0.5);
    float line = step(0.487,max(cell.x,cell.y));
    vec3 c = color * (1.0-grid*line*0.12);
    gl_FragColor = vec4(mix(c,vec3(0.68,0.80,0.85),fog),1.0);
}"#;
const UI_VERTEX: &str = r#"#version 100
attribute vec3 in_pos;
attribute vec3 in_normal;
attribute vec3 in_color;
attribute float in_style;
uniform mat4 matrix;
uniform vec3 eye;
varying vec3 color;
void main() { gl_Position = matrix * vec4(in_pos,1.0); color=in_color; }"#;
const UI_FRAGMENT: &str = r#"#version 100
precision mediump float;
varying vec3 color;
void main() { gl_FragColor=vec4(color,1.0); }"#;
