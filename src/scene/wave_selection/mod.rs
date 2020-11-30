use crate::core::scene::{Scene, SceneResult};
use crate::render::ui::{Gui, GuiContext};
use crate::resources::Resources;
use crate::save::get_wave_record;
use crate::scene::MainScene;
use bitflags::_core::time::Duration;
use glfw::{Key, WindowEvent};
use hecs::World;

#[derive(Default)]
pub struct WaveSelectionScene {
    selected: usize,
    possible: Vec<usize>,
    start: bool,
}

impl WaveSelectionScene {
    pub fn new(resources: &Resources) -> Self {
        let mut possible: Vec<usize> = vec![1];

        let record = get_wave_record(resources);
        if record >= 5 {
            let available = record / 5;
            for i in 0..available {
                possible.push(5 * (i + 1));
            }
        }

        Self {
            selected: 0,
            possible,
            start: false,
        }
    }
}

impl Scene<WindowEvent> for WaveSelectionScene {
    fn update(
        &mut self,
        _dt: Duration,
        _world: &mut World,
        _resources: &Resources,
    ) -> SceneResult<WindowEvent> {
        if self.start {
            SceneResult::ReplaceScene(Box::new(MainScene::new(true, self.possible[self.selected])))
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

        let center = gui_context.window_dim.to_vec2() / 2.0 - 100.0 * glam::Vec2::unit_y();

        gui.centered_label(center, "Choose starting wave".to_string());
        gui.centered_label(
            center + 20.0 * glam::Vec2::unit_y(),
            "Left/Right arrow to change, Enter to select".to_string(),
        );

        gui.centered_label(
            center + 60.0 * glam::Vec2::unit_y(),
            self.possible[self.selected].to_string(),
        );

        Some(gui)
    }

    fn process_input(&mut self, _world: &mut World, input: WindowEvent, _resources: &Resources) {
        match input {
            WindowEvent::Key(Key::Left, _, glfw::Action::Release, _) => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }

            WindowEvent::Key(Key::Right, _, glfw::Action::Release, _) => {
                if self.selected < self.possible.len() - 1 {
                    self.selected += 1;
                }
            }

            WindowEvent::Key(Key::Enter, _, glfw::Action::Release, _) => {
                self.start = true;
            }
            _ => (),
        }
    }
}
