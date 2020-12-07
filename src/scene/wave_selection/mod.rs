use crate::core::input::ser::{InputEvent, VirtualAction, VirtualKey};
use crate::core::scene::{Scene, SceneResult};
use crate::render::ui::{Gui, GuiContext};
use crate::resources::Resources;
use crate::save::get_wave_record;
use crate::scene::MainScene;
use core::time::Duration;
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

impl Scene for WaveSelectionScene {
    fn update(&mut self, _dt: Duration, _world: &mut World, _resources: &Resources) -> SceneResult {
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

    fn process_input(&mut self, _world: &mut World, input: InputEvent, _resources: &Resources) {
        match input {
            InputEvent::KeyEvent(VirtualKey::Left, VirtualAction::Release) => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }

            InputEvent::KeyEvent(VirtualKey::Right, VirtualAction::Release) => {
                if self.selected < self.possible.len() - 1 {
                    self.selected += 1;
                }
            }

            InputEvent::KeyEvent(VirtualKey::Enter, VirtualAction::Release) => {
                self.start = true;
            }
            _ => (),
        }
    }
}
