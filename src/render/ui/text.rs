use crate::core::colors::RgbaColor;
use crate::core::window::WindowDim;
use crate::resources::Resources;
use crate::{HEIGHT, WIDTH};
use glyph_brush::{rusttype::*, *};
use log::info;
use luminance::blending::{Blending, Equation, Factor};
use luminance::context::GraphicsContext;
use luminance::pipeline::{Pipeline, PipelineError, TextureBinding};
use luminance::pixel::{NormR8UI, NormUnsigned};
use luminance::render_state::RenderState;
use luminance::shader::{Program, Uniform};
use luminance::shading_gate::ShadingGate;
use luminance::tess::Mode;
use luminance::tess::Tess;
use luminance::texture::{Dim2, GenMipmaps, Sampler, Texture};
use luminance_derive::{Semantics, UniformInterface, Vertex};
use luminance_gl::GL33;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Semantics)]
pub enum VertexSemantics {
    #[sem(name = "left_top", repr = "[f32; 3]", wrapper = "VertexLeftTop")]
    LeftTop,

    #[sem(
        name = "right_bottom",
        repr = "[f32; 2]",
        wrapper = "VertexRightBottom"
    )]
    RightBottom,

    #[sem(name = "tex_left_top", repr = "[f32; 2]", wrapper = "TextureLeftTop")]
    TexLeftTop,

    #[sem(
        name = "tex_right_bottom",
        repr = "[f32; 2]",
        wrapper = "TextureRightBottom"
    )]
    TexRightBottom,

    #[sem(name = "color", repr = "[f32; 4]", wrapper = "TextColor")]
    Color,
}

#[allow(dead_code)]
#[repr(C)]
#[derive(Vertex, Copy, Debug, Clone)]
#[vertex(sem = "VertexSemantics", instanced = "true")]
pub struct Instance {
    left_top: VertexLeftTop,
    right_bottom: VertexRightBottom,
    tex_left_top: TextureLeftTop,
    tex_right_bottom: TextureRightBottom,
    color: TextColor,
}

#[derive(UniformInterface)]
pub struct ShaderInterface {
    pub tex: Uniform<TextureBinding<Dim2, NormUnsigned>>,
    //pub transform: Uniform<[[f32; 4]; 4]>,
}

#[derive(Debug, Clone)]
pub struct Text {
    pub content: String,
    pub font_size: f32,
}

/// X and Y coords between 0 and 1. (0,0) being the top-left corner and (1,1) bottom-right corner
pub struct ScreenPosition {
    pub x: f32,
    pub y: f32,
}

const VS: &'static str = include_str!("text-vs.glsl");
const FS: &'static str = include_str!("text-fs.glsl");

pub fn new_shader<B>(surface: &mut B) -> Program<GL33, VertexSemantics, (), ShaderInterface>
where
    B: GraphicsContext<Backend = GL33>,
{
    surface
        .new_shader_program::<VertexSemantics, (), ShaderInterface>()
        .from_strings(VS, None, None, FS)
        .expect("Program creation")
        .ignore_warnings()
}

pub struct TextRenderer<S>
where
    S: GraphicsContext<Backend = GL33>,
{
    projection: glam::Mat4,
    texture: Texture<S::Backend, Dim2, NormR8UI>,
    tess: Option<Tess<S::Backend, (), (), Instance>>,
    render_state: RenderState,
    shader: Program<S::Backend, VertexSemantics, (), ShaderInterface>,
}

impl<S> TextRenderer<S>
where
    S: GraphicsContext<Backend = GL33>,
{
    pub fn new(surface: &mut S, glyph_brush: &mut GlyphBrush<'static, Instance>) -> Self {
        let projection =
            glam::Mat4::orthographic_rh_gl(0.0, WIDTH as f32, 0.0, HEIGHT as f32, 1.0, -1.0);

        let render_state = RenderState::default()
            .set_blending(Blending {
                equation: Equation::Additive,
                src: Factor::SrcAlpha,
                dst: Factor::Zero,
            })
            .set_depth_test(None);
        let tex: Texture<S::Backend, Dim2, NormR8UI> = Texture::new(
            surface,
            [
                glyph_brush.texture_dimensions().0,
                glyph_brush.texture_dimensions().1,
            ],
            0,
            Sampler::default(),
        )
        .expect("luminance texture creation");

        Self {
            projection,
            texture: tex,
            tess: None,
            render_state,
            shader: new_shader(surface),
        }
    }

    pub fn prepare(
        &mut self,
        surface: &mut S,
        text_data: Vec<(Text, glam::Vec2, RgbaColor)>,
        glyph_brush: &mut GlyphBrush<'static, Instance>,
        resources: &Resources,
    ) {
        let window_dim = resources.fetch::<WindowDim>().unwrap();
        let width = window_dim.width as f32;
        let height = window_dim.height as f32;

        for (text, position, color) in text_data {
            // screen position is top-left origin
            let pos_x = position.x();
            let pos_y = position.y();
            info!("Will display text at {}/{}", pos_x, pos_y);

            let scale = Scale::uniform(text.font_size.round());
            glyph_brush.queue(Section {
                text: text.content.as_str(),
                scale,
                screen_position: (pos_x, pos_y),
                bounds: (width / 3.15, height),
                color: color.to_normalized(),
                layout: Layout::default()
                    .h_align(HorizontalAlign::Center)
                    .v_align(VerticalAlign::Bottom),
                ..Section::default()
            });
        }

        let action = glyph_brush
            .process_queued(
                |rect, tex_data| {
                    // Update part of gpu texture with new glyph alpha values
                    self.texture
                        .upload_part_raw(
                            GenMipmaps::No,
                            [rect.min.x as u32, rect.min.y as u32],
                            [rect.width() as u32, rect.height() as u32],
                            tex_data,
                        )
                        .expect("Cannot upload part of texture");
                },
                |vertex_data| to_vertex(width, height, vertex_data),
            )
            .unwrap();

        match action {
            BrushAction::Draw(v) => {
                let tess = surface
                    .new_tess()
                    .set_vertex_nb(4)
                    .set_instances(v)
                    .set_mode(Mode::TriangleStrip)
                    .build()
                    .unwrap();
                self.tess = Some(tess);
            }
            BrushAction::ReDraw => (),
        };
    }

    pub fn render(
        &mut self,
        pipeline: &Pipeline<S::Backend>,
        shd_gate: &mut ShadingGate<S::Backend>,
    ) -> Result<(), PipelineError> {
        let tex = &mut self.texture;
        let shader = &mut self.shader;
        let render_state = &self.render_state;
        let proj = self.projection.to_cols_array_2d();
        if let Some(tess) = self.tess.as_ref() {
            shd_gate.shade(shader, |mut iface, uni, mut rdr_gate| {
                let bound_tex = pipeline.bind_texture(tex)?;
                iface.set(&uni.tex, bound_tex.binding());
                //iface.set(&uni.transform, proj);
                rdr_gate.render(render_state, |mut tess_gate| tess_gate.render(tess))
            })?;
        }

        Ok(())
    }
}

#[inline]
fn to_vertex(
    width: f32,
    height: f32,
    glyph_brush::GlyphVertex {
        mut tex_coords,
        pixel_coords,
        bounds,
        color,
        z,
    }: glyph_brush::GlyphVertex,
) -> Instance {
    let gl_bounds = bounds;

    let mut gl_rect = Rect {
        min: point(pixel_coords.min.x as f32, pixel_coords.min.y as f32),
        max: point(pixel_coords.max.x as f32, pixel_coords.max.y as f32),
    };
    info!("GL_RECT = {:?}", gl_rect);

    // handle overlapping bounds, modify uv_rect to preserve texture aspect
    if gl_rect.max.x > gl_bounds.max.x {
        let old_width = gl_rect.width();
        gl_rect.max.x = gl_bounds.max.x;
        tex_coords.max.x = tex_coords.min.x + tex_coords.width() * gl_rect.width() / old_width;
    }
    if gl_rect.min.x < gl_bounds.min.x {
        let old_width = gl_rect.width();
        gl_rect.min.x = gl_bounds.min.x;
        tex_coords.min.x = tex_coords.max.x - tex_coords.width() * gl_rect.width() / old_width;
    }
    if gl_rect.max.y > gl_bounds.max.y {
        let old_height = gl_rect.height();
        gl_rect.max.y = gl_bounds.max.y;
        tex_coords.max.y = tex_coords.min.y + tex_coords.height() * gl_rect.height() / old_height;
    }
    if gl_rect.min.y < gl_bounds.min.y {
        let old_height = gl_rect.height();
        gl_rect.min.y = gl_bounds.min.y;
        tex_coords.min.y = tex_coords.max.y - tex_coords.height() * gl_rect.height() / old_height;
    }

    let to_view_space = |x: f32, y: f32| -> [f32; 2] {
        let pos_x = (x / width) * 2.0 - 1.0;
        let pos_y = (1.0 - y / height) * 2.0 - 1.0;
        [pos_x, pos_y]
    };

    let left_top = to_view_space(gl_rect.min.x, gl_rect.max.y);

    let v = Instance {
        left_top: VertexLeftTop::new([left_top[0], left_top[1], z]),
        right_bottom: VertexRightBottom::new(to_view_space(gl_rect.max.x, gl_rect.min.y)),
        tex_left_top: TextureLeftTop::new([tex_coords.min.x, tex_coords.max.y]),
        tex_right_bottom: TextureRightBottom::new([tex_coords.max.x, tex_coords.min.y]),
        color: TextColor::new(color),
    };

    info!("vertex -> {:?}", v);
    v
}
