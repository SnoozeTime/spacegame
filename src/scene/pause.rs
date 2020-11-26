//! Pause scene is when the player presses the escape button while playing. It will just bring
//! Some buttons to abandon or resume the game.

use crate::core::colors::RgbaColor;
use crate::core::scene::{Scene, SceneResult};
use crate::event::GameEvent;
use crate::render::ui::{Gui, GuiContext};
use crate::resources::Resources;
use crate::scene::main_menu::MainMenu;
use crate::ui::menu_button;
use bitflags::_core::time::Duration;
use glfw::{Key, WindowEvent};
use hecs::World;

#[derive(Default)]
pub struct PauseScene {
    resume: bool,
    go_to_menu: bool,
}

impl Scene<WindowEvent> for PauseScene {
    fn update(
        &mut self,
        _dt: Duration,
        _world: &mut World,
        _resources: &Resources,
    ) -> SceneResult<WindowEvent> {
        if self.resume {
            SceneResult::Pop
        } else if self.go_to_menu {
            SceneResult::ReplaceAll(Box::new(MainMenu::default()))
        } else {
            SceneResult::Noop
        }
    }

    fn prepare_gui(
        &mut self,
        _dt: Duration,
        _world: &mut World,
        _resources: &Resources,
        gui_context: &GuiContext,
    ) -> Option<Gui> {
        let mut gui = gui_context.new_frame();
        let window_dim = gui.window_dim.to_vec2();
        let anchor = glam::vec2(window_dim.x() / 2.0, window_dim.y() / 2.0);
        gui.panel(
            glam::vec2(0.0, 0.0),
            window_dim,
            RgbaColor::new(133, 133, 133, 133),
        );
        if menu_button("Resume", anchor, 32.0, &mut gui) {
            self.resume = true;
        }

        if menu_button(
            "Quit to Menu",
            anchor + 64.0 * glam::Vec2::unit_y(),
            32.0,
            &mut gui,
        ) {
            self.go_to_menu = true;
        }

        Some(gui)
    }

    fn process_input(&mut self, _world: &mut World, input: WindowEvent, _resources: &Resources) {
        if let WindowEvent::Key(Key::Escape, _0, glfw::Action::Press, _2) = input {
            self.resume = true;
        }
    }
}
