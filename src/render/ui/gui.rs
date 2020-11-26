use crate::core::colors::RgbaColor;
use crate::core::window::WindowDim;
use crate::render::ui::text::Text;
use crate::render::ui::{text, Button, DrawData, Panel, FONT_DATA};
use glfw::{Action, MouseButton, WindowEvent};
use glyph_brush::GlyphBrushBuilder;
use serde_derive::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

use glyph_brush::rusttype::Scale;
use glyph_brush::{GlyphBrush, GlyphCruncher, Layout, Section};

pub struct GuiContext {
    pub(crate) window_dim: WindowDim,
    pub(crate) mouse_pos: glam::Vec2,
    pub(crate) mouse_clicked: Vec<MouseButton>,
    pub(crate) style: Style,

    pub(crate) fonts: Rc<RefCell<GlyphBrush<'static, text::Instance>>>,
}

impl GuiContext {
    pub fn new(window_dim: WindowDim) -> Self {
        let fonts = GlyphBrushBuilder::using_font_bytes(FONT_DATA).build();

        Self {
            fonts: Rc::new(RefCell::new(fonts)),
            window_dim,
            mouse_pos: glam::Vec2::zero(),
            mouse_clicked: vec![],
            style: Style::default(),
        }
    }

    pub fn reset_inputs(&mut self) {
        self.mouse_clicked.clear();
    }

    pub fn process_event(&mut self, window_event: WindowEvent) {
        match window_event {
            WindowEvent::MouseButton(btn, Action::Press, _) => self.mouse_clicked.push(btn),
            WindowEvent::CursorPos(x, y) => {
                self.mouse_pos.set_x(x as f32);
                self.mouse_pos.set_y(y as f32);
            }
            _ => (),
        }
    }

    pub fn new_frame(&self) -> Gui {
        Gui::new(
            self.window_dim,
            self.mouse_pos,
            self.mouse_clicked.clone(),
            self.style,
            Rc::clone(&self.fonts),
        )
    }
}

pub struct Gui {
    pub(crate) draw_data: Vec<DrawData>,
    pub(crate) window_dim: WindowDim,
    pub(crate) mouse_pos: glam::Vec2,
    pub(crate) mouse_clicked: Vec<MouseButton>,
    pub(crate) style: Style,
    pub(crate) fonts: Rc<RefCell<GlyphBrush<'static, text::Instance>>>,
}

impl Gui {
    pub fn new(
        window_dim: WindowDim,
        mouse_pos: glam::Vec2,
        mouse_clicked: Vec<MouseButton>,
        style: Style,
        fonts: Rc<RefCell<GlyphBrush<'static, text::Instance>>>,
    ) -> Self {
        Self {
            draw_data: vec![],
            window_dim,
            mouse_clicked,
            mouse_pos,
            style,
            fonts,
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

    pub fn centered_label(&mut self, pos: glam::Vec2, text: String) {
        let bounds = self.text_bounds(text.as_str(), self.style.font_size);
        let real_pos = pos - bounds / 2.0;
        self.draw_data.push(DrawData::Text(
            Text {
                content: text,
                font_size: self.style.font_size,
                color: self.style.text_color,
                align: (HorizontalAlign::Left, VerticalAlign::Top),
            },
            real_pos,
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
        let mut btn = Button::new(text, pos);
        if let Some(dim) = dimensions {
            btn = btn.dimensions(dim);
        }

        btn.build(self)
    }

    pub fn text_bounds(&mut self, text: &str, font_size: f32) -> glam::Vec2 {
        let scale = Scale::uniform(font_size.round());
        let section = Section {
            text,
            scale,
            screen_position: (0.0, 0.0), //(text_position.x(), text_position.y()),
            bounds: (
                self.window_dim.width as f32 / 3.15,
                self.window_dim.height as f32,
            ),
            color: RgbaColor::new(0, 0, 0, 0).to_normalized(),
            layout: Layout::default()
                .h_align(self.style.button_text_align.0.into())
                .v_align(self.style.button_text_align.1.into()),
            ..Section::default()
        };

        let bounds = self
            .fonts
            .borrow_mut()
            .glyph_bounds(section)
            .expect("Text should have bounds");

        let height = bounds.height();
        let width = bounds.width();
        glam::vec2(width, height)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

impl Into<glyph_brush::HorizontalAlign> for HorizontalAlign {
    fn into(self) -> glyph_brush::HorizontalAlign {
        match self {
            Self::Center => glyph_brush::HorizontalAlign::Center,
            Self::Left => glyph_brush::HorizontalAlign::Left,
            Self::Right => glyph_brush::HorizontalAlign::Right,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
}

impl Into<glyph_brush::VerticalAlign> for VerticalAlign {
    fn into(self) -> glyph_brush::VerticalAlign {
        match self {
            Self::Top => glyph_brush::VerticalAlign::Top,
            Self::Center => glyph_brush::VerticalAlign::Center,
            Self::Bottom => glyph_brush::VerticalAlign::Bottom,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Style {
    /// background color for buttons
    pub button_bg_color: RgbaColor,
    /// background color for buttons that are hovered
    pub button_hover_bg_color: RgbaColor,
    /// color for text within buttons.
    pub button_text_color: RgbaColor,
    /// color for text when button is hovered
    pub button_hovered_text_color: RgbaColor,
    /// Horizontal and vertical align for text in a button
    pub button_text_align: (HorizontalAlign, VerticalAlign),
    /// text color
    pub text_color: RgbaColor,
    /// font size
    pub font_size: f32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            button_bg_color: RgbaColor::new(133, 133, 133, 133),
            button_hover_bg_color: RgbaColor::new(155, 155, 155, 155),
            button_text_color: RgbaColor::new(255, 255, 255, 255),
            button_hovered_text_color: RgbaColor::new(255, 255, 255, 255),
            button_text_align: (HorizontalAlign::Center, VerticalAlign::Center),
            text_color: RgbaColor::new(255, 255, 255, 255),
            font_size: 16.0,
        }
    }
}
