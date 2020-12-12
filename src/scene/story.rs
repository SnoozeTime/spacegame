use crate::core::scene::{Scene, SceneResult};
use crate::core::timer::Timer;
use crate::render::ui::{Gui, GuiContext};
use crate::resources::Resources;
use bitflags::_core::time::Duration;
use glfw::{Key, WindowEvent};
use hecs::World;

pub struct StoryScene<S: Scene<WindowEvent>> {
    sentences: Vec<String>,
    current_sentence: usize,
    timer_before_instruction: Timer,
    next_scene: Option<S>,
}

impl<S> StoryScene<S>
where
    S: Scene<WindowEvent> + 'static,
{
    pub fn new(sentences: Vec<String>, next_scene: S) -> Self {
        assert!(sentences.len() > 0);
        Self {
            sentences,
            current_sentence: 0,
            next_scene: Some(next_scene),
            timer_before_instruction: Timer::of_seconds(2.0),
        }
    }
}

impl<S> Scene<WindowEvent> for StoryScene<S>
where
    S: Scene<WindowEvent> + 'static,
{
    fn update(
        &mut self,
        dt: Duration,
        _world: &mut World,
        _resources: &Resources,
    ) -> SceneResult<WindowEvent> {
        self.timer_before_instruction.tick(dt);

        if self.current_sentence == self.sentences.len() {
            SceneResult::ReplaceScene(Box::new(self.next_scene.take().unwrap()))
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

        if let Some(sentence) = self.sentences.get(self.current_sentence) {
            gui.centered_label(window_dim / 2.0, sentence.clone());
        }

        if self.timer_before_instruction.finished() {
            gui.centered_label(
                glam::Vec2::new(window_dim.x / 2.0, window_dim.y - 60.0),
                "Press Enter to continue...".to_string(),
            );
        }

        Some(gui)
    }

    fn process_input(&mut self, _world: &mut World, input: WindowEvent, _resources: &Resources) {
        if let WindowEvent::Key(Key::Enter, _, glfw::Action::Press, _) = input {
            self.current_sentence += 1;
            self.timer_before_instruction.reset();
        }
    }
}
