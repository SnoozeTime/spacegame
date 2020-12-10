use crate::core::colors;
use crate::core::colors::RgbaColor;
use crate::core::scene::{Scene, SceneResult};
use crate::render::path::debug;
use crate::render::ui::{Gui, GuiContext};
use crate::resources::Resources;
use crate::ui::{disabled_menu_button, draw_cursor, menu_button};
use core::time::Duration;
use hecs::World;

pub struct DebugScene;

impl Scene for DebugScene {
    fn update(&mut self, _dt: Duration, _world: &mut World, resources: &Resources) -> SceneResult {
        debug::stroke_circle(
            resources,
            glam::vec2(100.0, 100.0),
            45.0,
            RgbaColor::new(255, 0, 0, 255),
        );

        SceneResult::Noop
    }

    fn prepare_gui(
        &mut self,
        _dt: Duration,
        _world: &mut World,
        _resources: &Resources,
        gui_context: &GuiContext,
    ) -> Option<Gui> {
        let mut gui = gui_context.new_frame();
        draw_cursor(&mut gui);

        gui.colored_label(glam::vec2(500.0, 500.0), "Bonjour".to_string(), colors::RED);
        let w = gui_context.window_dim.width as f32;
        let h = gui_context.window_dim.height as f32;
        // panel should take up 20% of the window width and 80% of the window height
        let panel_width = w * 0.2;
        let panel_height = h * 0.6;

        let anchor = glam::vec2(w / 2.0 - panel_width / 2.0, h / 2.0 - panel_height / 2.0);

        let mut gui = gui_context.new_frame();
        draw_cursor(&mut gui);

        disabled_menu_button(
            "Infinite Mode",
            anchor + 80.0 * glam::Vec2::unit_y(),
            48.0,
            &mut gui,
        );

        Some(gui)
    }
}
