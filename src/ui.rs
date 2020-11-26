//! Utilities to build the UI

use crate::core::colors::RgbaColor;
use crate::render::ui::{Button, Gui, HorizontalAlign, VerticalAlign};

pub(crate) fn menu_button(text: &str, position: glam::Vec2, font_size: f32, ui: &mut Gui) -> bool {
    Button::new(text.to_string(), position)
        .set_bg_color(RgbaColor::new(0, 0, 0, 0), RgbaColor::new(0, 0, 0, 0))
        .set_text_color(
            RgbaColor::from_hex("FFFFFFFF").unwrap(),
            RgbaColor::from_hex("01FFFFFF").unwrap(),
        )
        .set_font_size(font_size)
        .set_text_align(HorizontalAlign::Left, VerticalAlign::Top)
        .set_padding(0.0)
        .build(ui)
}
