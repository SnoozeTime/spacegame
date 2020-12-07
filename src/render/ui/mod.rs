use crate::render::ui::text::{Text, TextRenderer};
use crate::resources::Resources;
use glyph_brush::GlyphBrush;
use luminance::blending::{Blending, Equation, Factor};
use luminance::context::GraphicsContext;
use luminance::pipeline::PipelineError;
use luminance::render_state::RenderState;
use luminance::tess::Mode;
use luminance_derive::{Semantics, Vertex};
use luminance_front::{pipeline::Pipeline, shader::Program, shading_gate::ShadingGate, tess::Tess};

pub mod gui;
pub use gui::*;
pub mod text;
pub mod widgets;
pub use widgets::*;

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

pub fn new_shader(surface: &mut super::Context) -> Program<VertexSemantics, (), ()> {
    surface
        .new_shader_program::<VertexSemantics, (), ()>()
        .from_strings(VS, None, None, FS)
        .expect("Program creation")
        .ignore_warnings()
}

const FONT_DATA: &'static [u8] = include_bytes!("../../../assets/fonts/FFFFORWA.TTF");

pub struct UiRenderer {
    tesses: Vec<Tess<Vertex, u32>>,
    shader: Program<VertexSemantics, (), ()>,
    render_state: RenderState,
    text_renderer: TextRenderer,
}

pub enum DrawData {
    Vertices(Vec<Vertex>, Vec<u32>),
    Text(Text, glam::Vec2),
}

impl UiRenderer {
    pub fn new(surface: &mut super::Context, gui_context: &GuiContext) -> Self {
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

        Self {
            tesses: vec![],
            shader,
            render_state,
            text_renderer: TextRenderer::new(surface, &mut *gui_context.fonts.borrow_mut()),
        }
    }

    /// Recreate the texture
    pub fn prepare(
        &mut self,
        surface: &mut super::Context,
        gui: Option<Gui>,
        resources: &Resources,
        fonts: &mut GlyphBrush<'static, text::Instance>,
    ) {
        self.tesses.clear();

        if let Some(gui) = gui {
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
                    DrawData::Text(text, pos) => text_data.push((text, pos)),
                }
            }

            self.text_renderer
                .prepare(surface, text_data, fonts, resources);
        } else {
            self.text_renderer.tess = None;
        }
    }

    pub fn render(
        &mut self,
        pipeline: &Pipeline,
        shd_gate: &mut ShadingGate,
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
