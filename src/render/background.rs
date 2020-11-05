//! Background is a scrolling background

use crate::assets::sprite::SpriteAsset;
use crate::assets::{AssetManager, Handle};
use crate::core::camera::Camera;
use crate::core::window::WindowDim;
use crate::resources::Resources;
use luminance::blending::{Blending, Equation, Factor};
use luminance::context::GraphicsContext;
use luminance::pipeline::{Pipeline, PipelineError, TextureBinding};
use luminance::pixel::NormUnsigned;
use luminance::render_state::RenderState;
use luminance::shader::{Program, Uniform};
use luminance::shading_gate::ShadingGate;
use luminance::tess::{Mode, Tess};
use luminance::texture::Dim2;
use luminance_derive::UniformInterface;
use luminance_gl::GL33;

const VS: &'static str = include_str!("background-vs.glsl");
const FS: &'static str = include_str!("background-fs.glsl");

#[derive(UniformInterface)]
pub struct BackgroundUniform {
    offset_x: Uniform<f32>,
    offset_y: Uniform<f32>,
    tex: Uniform<TextureBinding<Dim2, NormUnsigned>>,
}

pub fn new_shader<B>(surface: &mut B) -> Program<GL33, (), (), BackgroundUniform>
where
    B: GraphicsContext<Backend = GL33>,
{
    surface
        .new_shader_program::<(), (), BackgroundUniform>()
        .from_strings(VS, None, None, FS)
        .expect("Program creation")
        .ignore_warnings()
}

pub struct BackgroundRenderer<S: GraphicsContext<Backend = GL33>> {
    render_st: RenderState,
    offset_x: f32,
    offset_y: f32,
    texture_handles: Vec<Handle>,
    tess: Tess<S::Backend, ()>,
    shader: Program<S::Backend, (), (), BackgroundUniform>,
}

impl<S> BackgroundRenderer<S>
where
    S: GraphicsContext<Backend = GL33>,
{
    pub fn new(surface: &mut S) -> Self {
        let render_st = RenderState::default()
            .set_depth_test(None)
            .set_blending(Blending {
                equation: Equation::Additive,
                src: Factor::SrcAlpha,
                dst: Factor::DstAlpha,
            });
        let tess = surface
            .new_tess()
            .set_vertex_nb(4)
            .set_mode(Mode::TriangleFan)
            .build()
            .expect("Tess creation");

        Self {
            texture_handles: [
                "starfield_2048.png",
                "starfield_729.png",
                "starfield_625.png",
            ]
            .iter()
            .map(|n| Handle(n.to_string()))
            .collect(),
            offset_x: 0.0,
            offset_y: 0.0,
            tess,
            render_st,
            shader: new_shader(surface),
        }
    }

    pub fn update(&mut self, world: &hecs::World, resources: &Resources) {
        if let Some((_, cam)) = world.query::<&Camera>().iter().next() {
            let dim = resources.fetch::<WindowDim>().unwrap();
            self.offset_x = cam.position.x() / (dim.width as f32);
            self.offset_y = cam.position.y() / (dim.height as f32);
        }
    }

    pub fn render(
        &mut self,
        pipeline: &Pipeline<S::Backend>,
        shd_gate: &mut ShadingGate<S::Backend>,
        textures: &mut AssetManager<S, SpriteAsset<S>>,
    ) -> Result<(), PipelineError> {
        let shader = &mut self.shader;
        let render_state = &self.render_st;
        let tess = &self.tess;
        let texture_handles = &self.texture_handles;
        let offset_x = self.offset_x;
        let offset_y = self.offset_y;
        shd_gate.shade(shader, |mut iface, uni, mut rdr_gate| {
            iface.set(&uni.offset_x, offset_x);
            iface.set(&uni.offset_y, offset_y);

            for h in texture_handles {
                if let Some(tex) = textures.get_mut(h) {
                    let mut res = Ok(());
                    tex.execute_mut(|tex| {
                        if let Some(tex) = tex.texture() {
                            let bound_tex = pipeline.bind_texture(tex);
                            match bound_tex {
                                Ok(bound_tex) => {
                                    iface.set(&uni.tex, bound_tex.binding());
                                    res = rdr_gate.render(render_state, |mut tess_gate| {
                                        tess_gate.render(tess)
                                    });
                                }
                                Err(e) => {
                                    res = Err(e);
                                }
                            }
                        }
                    });

                    res?;
                }
            }

            Ok(())
        })
    }
}
