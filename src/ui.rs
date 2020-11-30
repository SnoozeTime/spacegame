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

pub(crate) fn disabled_menu_button(
    text: &str,
    position: glam::Vec2,
    font_size: f32,
    ui: &mut Gui,
) -> bool {
    Button::new(text.to_string(), position)
        .set_bg_color(RgbaColor::new(0, 0, 0, 0), RgbaColor::new(0, 0, 0, 0))
        .set_text_color(
            RgbaColor::from_hex("808080FF").unwrap(),
            RgbaColor::from_hex("808080FF").unwrap(),
        )
        .set_font_size(font_size)
        .set_text_align(HorizontalAlign::Left, VerticalAlign::Top)
        .set_padding(0.0)
        .build(ui)
}

pub(crate) fn draw_cursor(gui: &mut Gui) {
    gui.panel(
        gui.mouse_pos - glam::vec2(2.0, 15.0),
        glam::vec2(4.0, 30.0),
        RgbaColor::new(255, 255, 255, 133),
    );
    gui.panel(
        gui.mouse_pos - glam::vec2(15.0, 2.0),
        glam::vec2(30.0, 4.0),
        RgbaColor::new(255, 255, 255, 133),
    );
}
