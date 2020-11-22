use crate::assets::shader::ShaderManager;
use crate::assets::Handle;
use crate::core::colors::RgbaColor;
use crate::core::transform::Transform;
use luminance::blending::{Blending, Equation, Factor};
use luminance::context::GraphicsContext;
use luminance::pipeline::{Pipeline, PipelineError, TextureBinding};
use luminance::pixel::NormUnsigned;
use luminance::render_state::RenderState;
use luminance::shader::Uniform;
use luminance::shading_gate::ShadingGate;
use luminance::tess::{Mode, Tess};
use luminance::texture::Dim2;
use luminance_derive::{Semantics, UniformInterface, Vertex};
use luminance_gl::GL33;
use std::time::Instant;

// Vertex definition
// -----------------
// Just position, texture coordinates and color for 2D. No need
// for normal, tangent...
// -----------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Semantics)]
pub enum VertexSemantics {
    #[sem(name = "position", repr = "[f32; 2]", wrapper = "VertexPosition")]
    Position,

    #[sem(name = "uv", repr = "[f32; 2]", wrapper = "TextureCoord")]
    TextureCoord,
    #[sem(name = "color", repr = "[f32; 4]", wrapper = "VertexColor")]
    Color,
}

#[allow(dead_code)]
#[repr(C)]
#[derive(Vertex, Copy, Debug, Clone)]
#[vertex(sem = "VertexSemantics")]
pub struct Vertex {
    /// Position of the vertex in 2D.
    position: VertexPosition,

    /// Texture coordinates for the vertex.
    uv: TextureCoord,

    /// Color for the vertex.
    color: VertexColor,
}

// Uniform definition
// ------------------
// Matrices to translate to view space, other useful uniforms such as timestamp, delta,
// and so on...
// --------------------------------------------------------------------------------------
#[allow(dead_code)]
#[derive(UniformInterface)]
pub struct ShaderUniform {
    /// PROJECTION matrix in MVP
    #[uniform(unbound, name = "u_projection")]
    projection: Uniform<[[f32; 4]; 4]>,
    /// VIEW matrix in MVP
    #[uniform(unbound, name = "u_view")]
    view: Uniform<[[f32; 4]; 4]>,
    /// MODEL matrix in MVP
    #[uniform(unbound, name = "u_model")]
    model: Uniform<[[f32; 4]; 4]>,
    /// Texture for the sprite.
    #[uniform(unbound, name = "u_tex_1")]
    tex_1: Uniform<TextureBinding<Dim2, NormUnsigned>>,
    /// true if should blink.
    #[uniform(unbound, name = "u_time")]
    time: Uniform<f32>,
}

pub enum Material {
    /// Will use the given vertex and fragment shaders for the mesh.
    Shader {
        vertex_shader_id: String,
        fragment_shader_id: String,
    },
    Texture {
        sprite_id: String,
    },
}

/// Render meshes with materials.
pub struct MeshRenderer<S>
where
    S: GraphicsContext<Backend = GL33>,
{
    tess: Tess<S::Backend, Vertex, u32>,
    /// used to send elapsed time to shader.
    creation_time: Instant,
}

pub struct MeshRender {
    pub enabled: bool,
    pub material: Material,
}

impl<S> MeshRenderer<S>
where
    S: GraphicsContext<Backend = GL33>,
{
    pub fn new(surface: &mut S) -> Self {
        let color = RgbaColor::new(255, 0, 0, 255).to_normalized();

        let (vertices, indices) = (
            vec![
                Vertex {
                    position: VertexPosition::new([-1.0, -1.0]),
                    uv: TextureCoord::new([0.0, 0.0]),
                    color: VertexColor::new(color),
                },
                Vertex {
                    position: VertexPosition::new([-1.0, 1.0]),
                    uv: TextureCoord::new([0.0, 1.0]),
                    color: VertexColor::new(color),
                },
                Vertex {
                    position: VertexPosition::new([1.0, 1.0]),
                    uv: TextureCoord::new([1.0, 1.0]),
                    color: VertexColor::new(color),
                },
                Vertex {
                    position: VertexPosition::new([1.0, -1.0]),
                    uv: TextureCoord::new([1.0, 0.0]),
                    color: VertexColor::new(color),
                },
            ],
            vec![0, 1, 2, 0, 2, 3],
        );

        let tess = surface
            .new_tess()
            .set_mode(Mode::Triangle)
            .set_indices(indices)
            .set_vertices(vertices)
            .build()
            .unwrap();

        Self {
            tess,
            creation_time: Instant::now(),
        }
    }
    pub fn render(
        &mut self,
        _pipeline: &Pipeline<S::Backend>,
        shd_gate: &mut ShadingGate<S::Backend>,
        proj_matrix: &glam::Mat4,
        view: &glam::Mat4,
        world: &hecs::World,
        shader_manager: &mut ShaderManager<S>,
    ) -> Result<(), PipelineError> {
        // let handle = Handle(("simple-vs.glsl".to_string(), "simple-fs.glsl".to_string()));

        let render_st = RenderState::default()
            .set_depth_test(None)
            .set_blending_separate(
                Blending {
                    equation: Equation::Additive,
                    src: Factor::SrcAlpha,
                    dst: Factor::SrcAlphaComplement,
                },
                Blending {
                    equation: Equation::Additive,
                    src: Factor::One,
                    dst: Factor::Zero,
                },
            );
        let elapsed = self.creation_time.elapsed().as_secs_f32();

        for (_, (t, render)) in world.query::<(&Transform, &MeshRender)>().iter() {
            if !render.enabled {
                continue;
            }

            if let Material::Shader {
                ref vertex_shader_id,
                ref fragment_shader_id,
            } = render.material
            {
                let model = t.to_model();

                let handle = Handle((vertex_shader_id.clone(), fragment_shader_id.clone()));
                if let Some(shader) = shader_manager.get_mut(&handle) {
                    if let Some(ret) = shader.execute_mut(|shader_asset| {
                        if let Some(ref mut shader) = shader_asset.shader {
                            shd_gate.shade(shader, |mut iface, uni, mut rdr_gate| {
                                iface.set(&uni.time, elapsed);
                                iface.set(&uni.projection, proj_matrix.to_cols_array_2d());
                                iface.set(&uni.view, view.to_cols_array_2d());
                                iface.set(&uni.model, model.to_cols_array_2d());
                                rdr_gate.render(&render_st, |mut tess_gate| {
                                    tess_gate.render(&self.tess)
                                })
                            })
                        } else {
                            Ok(())
                        }
                    }) {
                        ret?;
                    }
                } else {
                    shader_manager.load(handle.0);
                }
            }
        }

        Ok(())
    }
}
