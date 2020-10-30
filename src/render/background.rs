//! Background is a scrolling background

use luminance::backend::texture::Texture as TextureBackend;
use luminance::blending::{Blending, Equation, Factor};
use luminance::context::GraphicsContext;
use luminance::pipeline::{Pipeline, PipelineError, TextureBinding};
use luminance::pixel::{NormRGBA8UI, NormUnsigned};
use luminance::render_state::RenderState;
use luminance::shader::{Program, Uniform};
use luminance::shading_gate::ShadingGate;
use luminance::tess::{Mode, Tess};
use luminance::texture::{Dim2, GenMipmaps, MagFilter, MinFilter, Sampler, Texture, Wrap};
use luminance_derive::UniformInterface;
use luminance_gl::GL33;
use std::path::{Path, PathBuf};

const VS: &'static str = include_str!("background-vs.glsl");
const FS: &'static str = include_str!("background-fs.glsl");

#[derive(UniformInterface)]
pub struct BackgroundUniform {
    offset: Uniform<f32>,
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
    current_offset: f32,
    tex1: Texture<S::Backend, Dim2, NormRGBA8UI>,
    tex2: Texture<S::Backend, Dim2, NormRGBA8UI>,
    tex3: Texture<S::Backend, Dim2, NormRGBA8UI>,
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
        let base_path = std::env::var("ASSET_PATH").unwrap_or("".to_string());
        let tex1 = load_from_disk(
            surface,
            PathBuf::from(&base_path).join("assets/sprites/starfield_2048.png"),
        );
        let tex2 = load_from_disk(
            surface,
            PathBuf::from(&base_path).join("assets/sprites/starfield_729.png"),
        );
        let tex3 = load_from_disk(
            surface,
            PathBuf::from(&base_path).join("assets/sprites/starfield_625.png"),
        );

        Self {
            current_offset: 0.0,
            tess,
            tex1,
            tex2,
            tex3,
            render_st,
            shader: new_shader(surface),
        }
    }

    pub fn render(
        &mut self,
        pipeline: &Pipeline<S::Backend>,
        shd_gate: &mut ShadingGate<S::Backend>,
        shader: &mut Program<S::Backend, (), (), BackgroundUniform>,
    ) -> Result<(), PipelineError> {
        let tex1 = &mut self.tex1;
        let tex2 = &mut self.tex2;
        let tex3 = &mut self.tex3;
        let shader = &mut self.shader;
        let render_state = &self.render_st;
        let tess = &self.tess;
        self.current_offset += 0.001;
        let current_offset = self.current_offset;
        shd_gate.shade(shader, |mut iface, uni, mut rdr_gate| {
            iface.set(&uni.offset, current_offset);
            // FIRST TEXTURE
            let bound_tex = pipeline.bind_texture(tex1)?;
            iface.set(&uni.tex, bound_tex.binding());
            rdr_gate.render(render_state, |mut tess_gate| tess_gate.render(tess))?;

            // SECOND TEXTURE
            let bound_tex = pipeline.bind_texture(tex2)?;
            iface.set(&uni.tex, bound_tex.binding());
            rdr_gate.render(render_state, |mut tess_gate| tess_gate.render(tess))?;

            // THIRD TEXTURE
            let bound_tex = pipeline.bind_texture(tex3)?;
            iface.set(&uni.tex, bound_tex.binding());
            rdr_gate.render(render_state, |mut tess_gate| tess_gate.render(tess))
        })
    }
}

fn load_from_disk<B, P: AsRef<Path>>(
    surface: &mut B,
    path: P,
) -> Texture<B::Backend, Dim2, NormRGBA8UI>
where
    B: GraphicsContext,
    B::Backend: TextureBackend<Dim2, NormRGBA8UI>,
{
    let img = image::open(path).map(|img| img.flipv().to_rgba()).unwrap();
    let (width, height) = img.dimensions();
    let texels = img.into_raw();

    // create the luminance texture; the third argument is the number of mipmaps we want (leave it
    // to 0 for now) and the latest is the sampler to use when sampling the texels in the
    // shader (we’ll just use the default one)
    let mut tex = Texture::new(
        surface,
        [width, height],
        0,
        Sampler {
            wrap_r: Wrap::Repeat,
            wrap_s: Wrap::Repeat,
            wrap_t: Wrap::Repeat,
            min_filter: MinFilter::Nearest,
            mag_filter: MagFilter::Nearest,
            depth_comparison: None,
        },
    )
    .expect("luminance texture creation");

    // the first argument disables mipmap generation (we don’t care so far)
    tex.upload_raw(GenMipmaps::No, &texels).unwrap();

    tex
}
