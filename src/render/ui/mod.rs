use crate::core::colors::RgbaColor;
use crate::render::ui::text::{Text, TextRenderer};
use crate::{HEIGHT, WIDTH};
use glyph_brush::{GlyphBrush, GlyphBrushBuilder};
use luminance::blending::{Blending, Equation, Factor};
use luminance::context::GraphicsContext;
use luminance::pipeline::{Pipeline, PipelineError};
use luminance::render_state::RenderState;
use luminance::shader::Program;
use luminance::shading_gate::ShadingGate;
use luminance::tess::{Mode, Tess};
use luminance_derive::{Semantics, Vertex};
use luminance_gl::GL33;

pub mod text;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Semantics)]
pub enum VertexSemantics {
    #[sem(name = "position", repr = "[f32; 2]", wrapper = "Position")]
    Position,

    #[sem(name = "color", repr = "[f32; 4]", wrapper = "Color")]
    Color,
}

#[allow(dead_code)]
#[repr(C)]
#[derive(Vertex, Copy, Debug, Clone)]
#[vertex(sem = "VertexSemantics")]
pub struct Vertex {
    position: Position,
    color: Color,
}

const VS: &'static str = include_str!("ui-vs.glsl");
const FS: &'static str = include_str!("ui-fs.glsl");

pub fn new_shader<B>(surface: &mut B) -> Program<GL33, VertexSemantics, (), ()>
where
    B: GraphicsContext<Backend = GL33>,
{
    surface
        .new_shader_program::<VertexSemantics, (), ()>()
        .from_strings(VS, None, None, FS)
        .expect("Program creation")
        .ignore_warnings()
}

const FONT_DATA: &'static [u8] = include_bytes!("../../../assets/fonts/FFFFORWA.TTF");

pub struct UiRenderer<S>
where
    S: GraphicsContext<Backend = GL33>,
{
    tesses: Vec<Tess<S::Backend, Vertex, u32>>,
    shader: Program<S::Backend, VertexSemantics, (), ()>,
    render_state: RenderState,

    fonts: GlyphBrush<'static, text::Instance>,
    text_renderer: TextRenderer<S>,
}

pub struct Panel {
    anchor: glam::Vec2,
    dimensions: glam::Vec2,
    color: RgbaColor,
}

pub enum DrawData {
    Vertices(Vec<Vertex>, Vec<u32>),
    Text(Text, glam::Vec2, RgbaColor),
}

pub struct Gui {
    draw_data: Vec<DrawData>,
}

impl Gui {
    pub fn new() -> Self {
        Self { draw_data: vec![] }
    }
    pub fn panel(&mut self, pos: glam::Vec2, dimensions: glam::Vec2, color: RgbaColor) {
        let (vertices, indices) = Panel {
            anchor: pos,
            dimensions,
            color,
        }
        .vertices();
        self.draw_data.push(DrawData::Vertices(vertices, indices));
    }

    pub fn label(&mut self, pos: glam::Vec2, text: Text, color: RgbaColor) {
        self.draw_data.push(DrawData::Text(text, pos, color));
    }
}

impl Panel {
    fn vertices(&self) -> (Vec<Vertex>, Vec<u32>) {
        let top_left = glam::vec2(
            -1.0 + self.anchor.x() / WIDTH as f32,
            1.0 - self.anchor.y() / HEIGHT as f32,
        );
        let dim = glam::vec2(
            self.dimensions.x() / WIDTH as f32,
            self.dimensions.y() / HEIGHT as f32,
        );
        let top_right = top_left + dim.x() * glam::Vec2::unit_x();
        let bottom_right =
            top_left + dim.x() * glam::Vec2::unit_x() - dim.y() * glam::Vec2::unit_y();
        let bottom_left = top_left - dim.y() * glam::Vec2::unit_y();

        let color = self.color.to_normalized();
        (
            vec![
                Vertex {
                    position: Position::new(bottom_left.into()),
                    color: Color::new(color),
                },
                Vertex {
                    position: Position::new(top_left.into()),
                    color: Color::new(color),
                },
                Vertex {
                    position: Position::new(top_right.into()),
                    color: Color::new(color),
                },
                Vertex {
                    position: Position::new(bottom_right.into()),
                    color: Color::new(color),
                },
            ],
            vec![0, 1, 2, 0, 2, 3],
        )
    }
}

impl<S> UiRenderer<S>
where
    S: GraphicsContext<Backend = GL33>,
{
    pub fn new(surface: &mut S) -> Self {
        let shader = new_shader(surface);

        let render_state = RenderState::default()
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

        let mut fonts = GlyphBrushBuilder::using_font_bytes(FONT_DATA).build();

        Self {
            tesses: vec![],
            shader,
            render_state,
            text_renderer: TextRenderer::new(surface, &mut fonts),
            fonts,
        }
    }

    /// Recreate the texture
    pub fn prepare(&mut self, surface: &mut S, gui: Gui) {
        self.tesses.clear();

        let mut text_data = vec![];
        for draw_data in gui.draw_data {
            match draw_data {
                DrawData::Vertices(vertices, indices) => {
                    let tess = surface
                        .new_tess()
                        .set_mode(Mode::Triangle)
                        .set_indices(indices)
                        .set_vertices(vertices)
                        .build()
                        .unwrap();
                    self.tesses.push(tess);
                }
                DrawData::Text(text, pos, color) => text_data.push((text, pos, color)),
            }
        }

        self.text_renderer
            .prepare(surface, text_data, &mut self.fonts);
    }

    pub fn render(
        &mut self,
        pipeline: &Pipeline<S::Backend>,
        shd_gate: &mut ShadingGate<S::Backend>,
    ) -> Result<(), PipelineError> {
        let tesses = &self.tesses;
        let render_state = &self.render_state;

        for tess in tesses {
            shd_gate.shade(&mut self.shader, |_iface, _uni, mut rdr_gate| {
                rdr_gate.render(render_state, |mut tess_gate| tess_gate.render(tess))
            })?;
        }

        self.text_renderer.render(pipeline, shd_gate)
    }
}
