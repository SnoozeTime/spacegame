use crate::core::colors::RgbaColor;
use crate::core::window::WindowDim;
use crate::render::ui::Gui;
use glfw::{Action, MouseButton, WindowEvent};
use serde_derive::{Deserialize, Serialize};

pub struct GuiContext {
    pub(crate) window_dim: WindowDim,
    pub(crate) mouse_pos: glam::Vec2,
    pub(crate) mouse_clicked: Vec<MouseButton>,
    pub(crate) style: Style,
}

impl GuiContext {
    pub fn new(window_dim: WindowDim) -> Self {
        Self {
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
        )
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
            text_color: RgbaColor::new(0, 0, 0, 255),
            font_size: 16.0,
        }
    }
}
