use crate::core::colors::RgbaColor;
use crate::core::scene::{Scene, SceneResult};
use crate::render::ui::gui::GuiContext;
use crate::render::ui::Gui;
use crate::resources::Resources;
use crate::scene::MainScene;
use bitflags::_core::time::Duration;
use hecs::World;

#[derive(Default)]
pub struct MainMenu {
    does_start: bool,
}

impl Scene for MainMenu {
    fn update(&mut self, _dt: Duration, _world: &mut World, _resources: &Resources) -> SceneResult {
        if self.does_start {
            SceneResult::ReplaceScene(Box::new(MainScene::new()))
        } else {
            SceneResult::Noop
        }
    }

    fn prepare_gui(
        &mut self,
        _dt: Duration,
        _world: &mut World,
        resources: &Resources,
    ) -> Option<Gui> {
        let gui_context = resources.fetch::<GuiContext>().unwrap();

        let w = gui_context.window_dim.width as f32;
        let h = gui_context.window_dim.height as f32;
        // panel should take up 20% of the window width and 80% of the window height
        let panel_width = w * 0.2;
        let panel_height = h * 0.8;

        let anchor = glam::vec2(w / 2.0 - panel_width / 2.0, h / 2.0 - panel_height / 2.0);

        let mut gui = gui_context.new_frame();
        gui.panel(
            anchor,
            glam::vec2(panel_width, panel_height),
            RgbaColor::new(60, 60, 60, 150),
        );
        if gui.button(anchor, None, "Start".to_string()) {
            self.does_start = true;
        }

        Some(gui)
    }
}
