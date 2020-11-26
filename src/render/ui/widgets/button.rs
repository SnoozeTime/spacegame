use crate::core::colors::RgbaColor;
use crate::render::ui::gui::{HorizontalAlign, VerticalAlign};
use crate::render::ui::text::Text;
use crate::render::ui::{DrawData, Gui, Panel};
use glfw::MouseButton;
use glyph_brush::rusttype::Scale;
use glyph_brush::{GlyphCruncher, Layout, Section};

pub struct Button {
    /// Text of the button
    text: String,
    /// dimension of the button. If none, will use enough space for the text
    dimensions: Option<glam::Vec2>,
    /// top-left corner for the button
    anchor: glam::Vec2,
    /// Override default style
    text_color: Option<RgbaColor>,
    hover_text_color: Option<RgbaColor>,
    background_color: Option<RgbaColor>,
    hover_background_color: Option<RgbaColor>,
    font_size: Option<f32>,
    text_align: Option<(HorizontalAlign, VerticalAlign)>,
    padding: Option<f32>,
}

impl Button {
    pub fn new(text: String, position: glam::Vec2) -> Self {
        Self {
            text,
            dimensions: None,
            anchor: position,
            text_color: None,
            hover_text_color: None,
            background_color: None,
            hover_background_color: None,
            font_size: None,
            text_align: None,
            padding: None,
        }
    }

    pub fn dimensions(mut self, dim: glam::Vec2) -> Self {
        self.dimensions = Some(dim);
        self
    }

    pub fn set_font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }

    pub fn set_bg_color(mut self, color: RgbaColor, hover_color: RgbaColor) -> Self {
        self.background_color = Some(color);
        self.hover_background_color = Some(hover_color);
        self
    }

    pub fn set_text_color(mut self, color: RgbaColor, hover_color: RgbaColor) -> Self {
        self.text_color = Some(color);
        self.hover_text_color = Some(hover_color);
        self
    }

    pub fn set_text_align(
        mut self,
        horizontal_align: HorizontalAlign,
        vertical_align: VerticalAlign,
    ) -> Self {
        self.text_align = Some((horizontal_align, vertical_align));
        self
    }

    pub fn set_padding(mut self, padding: f32) -> Self {
        self.padding = Some(padding);
        self
    }

    fn text_color(&self, ui: &Gui, is_above: bool) -> RgbaColor {
        if is_above {
            self.hover_text_color
                .unwrap_or(ui.style.button_hovered_text_color)
        } else {
            self.text_color.unwrap_or(ui.style.button_text_color)
        }
    }

    fn background_color(&self, ui: &Gui, is_above: bool) -> RgbaColor {
        if is_above {
            self.hover_background_color
                .unwrap_or(ui.style.button_hover_bg_color)
        } else {
            self.background_color.unwrap_or(ui.style.button_bg_color)
        }
    }

    pub fn build(self, ui: &mut Gui) -> bool {
        //let padding = self.padding.unwrap_or(10.0);
        let font_size = self.font_size.unwrap_or(ui.style.font_size);
        let text_position = self.anchor; // + glam::Vec2::unit_y() * font_size;

        let text_align = self.text_align.unwrap_or(ui.style.button_text_align);

        let dimensions = if let Some(dimension) = self.dimensions {
            dimension
        } else {
            ui.text_bounds(self.text.as_str(), font_size)
        };

        let mouse_pos_rel = ui.mouse_pos - self.anchor;
        let is_above = mouse_pos_rel.x() >= 0.0
            && mouse_pos_rel.x() < dimensions.x()
            && mouse_pos_rel.y() >= 0.0
            && mouse_pos_rel.y() <= dimensions.y();
        let color = self.background_color(ui, is_above);
        let text_color = self.text_color(ui, is_above);
        let (vertices, indices) = Panel {
            anchor: self.anchor,
            dimensions,
            color,
        }
        .vertices(ui.window_dim);

        ui.draw_data.push(DrawData::Vertices(vertices, indices));

        //let horizontal_align = self.text_align.unwrap_or(ui.style.button_text_align).0;
        ui.draw_data.push(DrawData::Text(
            Text {
                content: self.text,
                font_size,
                color: text_color,
                align: text_align,
            },
            text_position,
        ));

        if ui.mouse_clicked.contains(&MouseButton::Button1) {
            return is_above;
        }

        false
    }
}
