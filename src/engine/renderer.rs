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
        Self {
            ctx,
            world_pipeline,
            ui_pipeline,
            stream,
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
        self.ctx
            .begin_default_pass(PassAction::clear_color(SKY[0], SKY[1], SKY[2], 1.));
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
    pub fn finish(&mut self) {
        self.ctx.end_render_pass();
        self.ctx.commit_frame();
    }

    /// Capture the actual OpenGL framebuffer for automated renderer smoke tests.
    pub fn capture(&self, path: &str, width: usize, height: usize) -> std::io::Result<()> {
        use miniquad::graphics::raw_gl::*;
        use std::io::Write;
        let mut pixels = vec![0u8; width * height * 4];
        // SAFETY: the active GL context owns the framebuffer; the allocated RGBA
        // buffer has exactly width * height * 4 bytes and remains live for the call.
        unsafe {
            glReadPixels(
                0,
                0,
                width as i32,
                height as i32,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                pixels.as_mut_ptr().cast(),
            );
        }
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
