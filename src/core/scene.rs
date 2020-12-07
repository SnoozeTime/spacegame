use crate::core::input::ser::InputEvent;
use crate::event::GameEvent;
use crate::render::ui::gui::GuiContext;
use crate::render::ui::Gui;
use crate::resources::Resources;
use hecs::World;
use std::time::Duration;

/// The stack will keep track of the states in the game.
/// The top of the stack will be used for the update loop. The states below
/// are still kept in memory so to go back to a previous state, you just have
/// to pop the stack.
pub struct SceneStack {
    states: Vec<Box<dyn Scene>>,
}

impl Default for SceneStack {
    fn default() -> Self {
        Self { states: vec![] }
    }
}

pub enum SceneResult {
    ReplaceScene(Box<dyn Scene>),
    Push(Box<dyn Scene>),
    Pop,
    /// Remove all existing scenes and create the new one.
    ReplaceAll(Box<dyn Scene>),
    Noop,
}

impl SceneStack {
    pub fn apply_result(
        &mut self,
        res: SceneResult,
        world: &mut hecs::World,
        resources: &mut Resources,
    ) {
        match res {
            SceneResult::ReplaceScene(state) => self.replace(state, world, resources),
            SceneResult::Push(state) => self.push(state, world, resources),
            SceneResult::Pop => {
                self.pop(world);
            }
            SceneResult::ReplaceAll(state) => {
                while !self.states.is_empty() {
                    self.pop(world);
                }
                self.push(state, world, resources);
            }
            SceneResult::Noop => (),
        }
    }

    /// Add a state to the game. Will be used for updating.
    ///
    /// The callback on_enter will be executed for the new state.
    pub fn push(
        &mut self,
        state: Box<dyn Scene>,
        world: &mut hecs::World,
        resources: &mut Resources,
    ) {
        if let Some(current) = self.states.last_mut() {
            current.on_exit();
        }

        self.states.push(state);
        if let Some(current) = self.states.last_mut() {
            current.on_create(world, resources);
        }
    }

    /// Remove the current state and execute its exit callback.
    pub fn pop(&mut self, world: &mut hecs::World) -> Option<Box<dyn Scene>> {
        if let Some(mut s) = self.states.pop() {
            s.on_destroy(world);
            if let Some(current) = self.states.last() {
                current.on_enter();
            }
            Some(s)
        } else {
            None
        }
    }

    /// Replace the current state.
    pub fn replace(
        &mut self,
        state: Box<dyn Scene>,
        world: &mut hecs::World,
        resources: &mut Resources,
    ) {
        if let Some(mut s) = self.states.pop() {
            s.on_destroy(world);
        }
        self.states.push(state);
        if let Some(current) = self.states.last_mut() {
            current.on_create(world, resources);
        }
    }

    /// Get the current state as a mut reference.
    #[allow(clippy::borrowed_box)]
    pub fn current_mut(&mut self) -> Option<&mut Box<dyn Scene>> {
        self.states.last_mut()
    }
}

pub trait Scene {
    /// WIll be called when the state is added to the state stack.
    fn on_create(&mut self, _world: &mut hecs::World, _resources: &mut Resources) {
        info!("Create state");
    }

    /// Will be called when the state is removed from the state stack.
    fn on_destroy(&mut self, _world: &mut hecs::World) {
        info!("Destroy state");
    }

    /// Will be called when the state becomes active. This is called
    /// on stack.pop
    ///
    /// Careful, this is not call on stack.push. Use the on_create callback instead.
    fn on_enter(&self) {
        info!("Enter state");
    }

    /// Will be called when the state becomes inactive. This is called on
    /// stack.push
    fn on_exit(&self) {
        info!("Exit state");
    }

    //fn on_new_world(&mut self);

    /// Update gameplay systems.
    fn update(&mut self, dt: Duration, world: &mut World, resources: &Resources) -> SceneResult;

    fn prepare_gui(
        &mut self,
        _dt: Duration,
        _world: &mut World,
        _resources: &Resources,
        _gui_context: &GuiContext,
    ) -> Option<Gui> {
        None
    }

    /// React to game events.
    fn process_event(&mut self, _world: &mut World, _ev: GameEvent, _resources: &Resources) {}

    /// Process input from keyboard/mouse
    fn process_input(&mut self, _world: &mut World, _input: InputEvent, _resources: &Resources) {}
}
