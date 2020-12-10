use crate::core::colors::RgbaColor;
use crate::core::window::WindowDim;
use crate::render::ui::gui::{HorizontalAlign, VerticalAlign};
use crate::render::Context;
use crate::resources::Resources;
use glyph_brush::rusttype::*;
use glyph_brush::BrushError::TextureTooSmall;
use glyph_brush::{BrushAction, GlyphBrush, Layout, Section};
use luminance::blending::{Blending, Equation, Factor};
use luminance::context::GraphicsContext;
use luminance::pipeline::{PipelineError, TextureBinding};
use luminance::pixel::{NormR8UI, NormUnsigned};
use luminance::render_state::RenderState;
use luminance::shader::Uniform;
use luminance::tess::Mode;
use luminance::texture::{Dim2, GenMipmaps, Sampler};
use luminance_derive::{Semantics, UniformInterface, Vertex};
use luminance_front::{
    pipeline::Pipeline, shader::Program, shading_gate::ShadingGate, tess::Tess, texture::Texture,
};

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
    pub color: RgbaColor,
    pub align: (HorizontalAlign, VerticalAlign),
}

/// X and Y coords between 0 and 1. (0,0) being the top-left corner and (1,1) bottom-right corner
pub struct ScreenPosition {
    pub x: f32,
    pub y: f32,
}

const VS: &'static str = include_str!("text-vs.glsl");
const FS: &'static str = include_str!("text-fs.glsl");

pub fn new_shader(surface: &mut Context) -> Program<VertexSemantics, (), ShaderInterface> {
    surface
        .new_shader_program::<VertexSemantics, (), ShaderInterface>()
        .from_strings(VS, None, None, FS)
        .expect("Program creation")
        .ignore_warnings()
}

pub struct TextRenderer {
    texture: Texture<Dim2, NormR8UI>,
    pub(crate) tess: Option<Tess<(), (), Instance>>,
    render_state: RenderState,
    shader: Program<VertexSemantics, (), ShaderInterface>,
}

impl TextRenderer {
    pub fn new(surface: &mut Context, glyph_brush: &mut GlyphBrush<'static, Instance>) -> Self {
        let mut render_state = RenderState::default().set_depth_test(None);

        if cfg!(target_arch = "wasm32") {
            render_state = render_state.set_blending(Blending {
                equation: Equation::Additive,
                src: Factor::One,
                dst: Factor::SrcAlphaComplement,
            });
        }

        if cfg!(not(target_arch = "wasm32")) {
            render_state = render_state.set_blending(Blending {
                equation: Equation::Additive,
                src: Factor::SrcAlpha,
                dst: Factor::Zero,
            });
        }

        let tex: Texture<Dim2, NormR8UI> = Texture::new(
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
            texture: tex,
            tess: None,
            render_state,
            shader: new_shader(surface),
        }
    }

    pub fn prepare(
        &mut self,
        surface: &mut Context,
        text_data: Vec<(Text, glam::Vec2)>,
        glyph_brush: &mut GlyphBrush<'static, Instance>,
        resources: &Resources,
    ) {
        let window_dim = resources.fetch::<WindowDim>().unwrap();
        let width = window_dim.width as f32;
        let height = window_dim.height as f32;

        //
        // glyph_brush.pixel_bounds()

        for (text, position) in text_data {
            // screen position is top-left origin
            let pos_x = position.x();
            let pos_y = position.y();
            debug!("Will display text at {}/{}", pos_x, pos_y);

            let scale = Scale::uniform(text.font_size.round());
            glyph_brush.queue(Section {
                text: text.content.as_str(),
                scale,
                screen_position: (pos_x, pos_y),
                bounds: (width / 3.15, height),
                color: text.color.to_normalized(),
                layout: Layout::default()
                    .h_align(text.align.0.into())
                    .v_align(text.align.1.into()),
                ..Section::default()
            });
        }

        let action = glyph_brush.process_queued(
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
        );

        if let Err(e) = action {
            let TextureTooSmall { suggested } = e;
            glyph_brush.resize_texture(suggested.0, suggested.1);
            return;
        }
        let action = action.unwrap();
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
        pipeline: &Pipeline,
        shd_gate: &mut ShadingGate,
    ) -> Result<(), PipelineError> {
        let tex = &mut self.texture;
        let shader = &mut self.shader;
        let render_state = &self.render_state;
        if let Some(tess) = self.tess.as_ref() {
            shd_gate.shade(shader, |mut iface, uni, mut rdr_gate| {
                let bound_tex = pipeline.bind_texture(tex)?;
                iface.set(&uni.tex, bound_tex.binding());
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
    debug!("GL_RECT = {:?}", gl_rect);

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

    debug!("vertex -> {:?}", v);
    v
}
