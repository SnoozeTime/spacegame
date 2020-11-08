use crate::core::colors::RgbaColor;
use crate::core::input::Input;
use crate::core::window::WindowDim;
use crate::gameplay::Action::MoveUp;
use crate::render::ui::gui::{HorizontalAlign, Style, VerticalAlign};
use crate::render::ui::text::{Text, TextRenderer};
use crate::resources::Resources;
use crate::{HEIGHT, WIDTH};
use glfw::MouseButton;
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

pub mod gui;
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
    Text(Text, glam::Vec2),
}

pub struct Gui {
    draw_data: Vec<DrawData>,
    window_dim: WindowDim,
    mouse_pos: glam::Vec2,
    mouse_clicked: Vec<MouseButton>,
    style: Style,
}

impl Gui {
    pub fn new(
        window_dim: WindowDim,
        mouse_pos: glam::Vec2,
        mouse_clicked: Vec<MouseButton>,
        style: Style,
    ) -> Self {
        Self {
            draw_data: vec![],
            window_dim,
            mouse_clicked,
            mouse_pos,
            style,
        }
    }
    pub fn panel(&mut self, pos: glam::Vec2, dimensions: glam::Vec2, color: RgbaColor) {
        let (vertices, indices) = Panel {
            anchor: pos,
            dimensions,
            color,
        }
        .vertices(self.window_dim);
        self.draw_data.push(DrawData::Vertices(vertices, indices));
    }

    pub fn label(&mut self, pos: glam::Vec2, text: String) {
        self.draw_data.push(DrawData::Text(
            Text {
                content: text,
                font_size: self.style.font_size,
                color: self.style.text_color,
                align: (HorizontalAlign::Left, VerticalAlign::Top),
            },
            pos,
        ));
    }
    pub fn colored_label(&mut self, pos: glam::Vec2, text: String, color: RgbaColor) {
        self.draw_data.push(DrawData::Text(
            Text {
                content: text,
                font_size: self.style.font_size,
                color,
                align: (HorizontalAlign::Left, VerticalAlign::Top),
            },
            pos,
        ));
    }

    pub fn button(
        &mut self,
        pos: glam::Vec2,
        dimensions: Option<glam::Vec2>,
        text: String,
    ) -> bool {
        let padding = 10.0;
        let dimensions = if let Some(dimension) = dimensions {
            dimension
        } else {
            let height = padding * 2.0 + self.style.font_size;
            let width = padding * 2.0 + text.len() as f32 * self.style.font_size * 0.8;
            glam::vec2(width, height)
        };

        let mouse_pos_rel = self.mouse_pos - pos;
        let is_above = mouse_pos_rel.x() >= 0.0
            && mouse_pos_rel.x() < dimensions.x()
            && mouse_pos_rel.y() >= 0.0
            && mouse_pos_rel.y() <= dimensions.y();

        let (color, text_color) = if is_above {
            (
                self.style.button_hover_bg_color,
                self.style.button_hovered_text_color,
            )
        } else {
            (self.style.button_bg_color, self.style.button_text_color)
        };

        let (vertices, indices) = Panel {
            anchor: pos,
            dimensions,
            color,
        }
        .vertices(self.window_dim);

        self.draw_data.push(DrawData::Vertices(vertices, indices));
        self.draw_data.push(DrawData::Text(
            Text {
                content: text,
                font_size: self.style.font_size,
                color: text_color,
                align: self.style.button_text_align,
            },
            pos + glam::Vec2::unit_y() * self.style.font_size
                + dimensions.x() / 2.0 * glam::Vec2::unit_x(),
        ));

        if self.mouse_clicked.contains(&MouseButton::Button1) {
            return is_above;
        }

        false
    }
}

impl Panel {
    fn vertices(&self, window_dim: WindowDim) -> (Vec<Vertex>, Vec<u32>) {
        info!("WindowDim -> {:#?}", window_dim);

        let w = window_dim.width as f32;
        let h = window_dim.height as f32;
        let x = (self.anchor.x() / w) * 2.0 - 1.0;
        let y = (1.0 - self.anchor.y() / h) * 2.0 - 1.0;
        let top_left = glam::vec2(x, y);
        info!("Anchor -> {:#?}", self.anchor);
        info!("Top left -> {:#?}", top_left);
        let dim = glam::vec2(2.0 * self.dimensions.x() / w, 2.0 * self.dimensions.y() / h);
        info!("Dimensions = {:?} => {:?}", self.dimensions, dim);
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
    pub fn prepare(&mut self, surface: &mut S, gui: Gui, resources: &Resources) {
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
                DrawData::Text(text, pos) => text_data.push((text, pos)),
            }
        }

        self.text_renderer
            .prepare(surface, text_data, &mut self.fonts, resources);
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
