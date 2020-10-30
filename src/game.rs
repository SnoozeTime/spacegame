use crate::core::input::{Input, InputAction};
use crate::event::GameEvent;
use crate::gameplay::delete::GarbageCollector;
use crate::render::Renderer;
use crate::resources::Resources;
use crate::scene::{Scene, SceneStack};
use glfw::{Action, Context, Key, WindowEvent};
use log::info;
use luminance_glfw::GlfwSurface;
use shrev::{EventChannel, ReaderId};
use std::any::Any;
use std::marker::PhantomData;
use std::thread;
use std::time::{Duration, Instant};

/// GameBuilder is used to create a new game. Game struct has a lot of members that do not need to be
/// exposed so gamebuilder provides a simpler way to get started.
pub struct GameBuilder<'a, A> {
    surface: &'a mut GlfwSurface,
    scene: Option<Box<dyn Scene>>,
    resources: Resources,
    phantom: PhantomData<A>,
}

impl<'a, A> GameBuilder<'a, A>
where
    A: InputAction + 'static,
{
    pub fn new(surface: &'a mut GlfwSurface) -> Self {
        // resources will need at least an event channel and an input
        let mut resources = Resources::default();
        let chan: EventChannel<GameEvent> = EventChannel::new();
        resources.insert(chan);

        // Need some input :D
        let input: Input<A> = Input::new();
        resources.insert(input);

        // and some asset manager;
        resources.insert(crate::assets::load_sprites(surface));

        Self {
            surface,
            scene: None,
            resources,
            phantom: PhantomData::default(),
        }
    }

    /// Set up the first scene.
    pub fn for_scene(mut self, scene: Box<dyn Scene>) -> Self {
        self.scene = Some(scene);
        self
    }

    /// Add custom resources.
    pub fn with_resource<T: Any>(mut self, r: T) -> Self {
        self.resources.insert(r);
        self
    }

    pub fn build(mut self) -> Game<'a, A> {
        let renderer = Renderer::new(self.surface, &mut self.resources);

        let mut world = hecs::World::new();

        let scene_stack = {
            let mut scenes = SceneStack::default();
            if let Some(scene) = self.scene {
                scenes.push(scene, &mut world, &mut self.resources);
            }
            scenes
        };

        let rdr_id = {
            let mut chan = self
                .resources
                .fetch_mut::<EventChannel<GameEvent>>()
                .unwrap();
            chan.register_reader()
        };

        let garbage_collector = GarbageCollector::new(&mut self.resources);

        Game {
            surface: self.surface,
            renderer,
            scene_stack,
            world,
            resources: self.resources,
            rdr_id,
            garbage_collector,
            phantom: self.phantom,
        }
    }
}

/// Struct that holds the game state and systems.
///
/// # Lifetime requirement:
/// The opengl context is held in GlfwSurface. This is a mutable reference here as we do not want the
/// context to be dropped at the same time as the systems. If it is dropped before, then releasing GPU
/// resources will throw a segfault.
///
/// # Generic parameters:
/// - A: Action that is derived from the inputs. (e.g. Move Left)
///
pub struct Game<'a, A> {
    /// for drawing stuff
    surface: &'a mut GlfwSurface,
    renderer: Renderer<GlfwSurface>,

    /// All the scenes. Current scene will be used in the main loop.
    scene_stack: SceneStack,

    /// Resources (assets, inputs...)
    resources: Resources,

    /// Current entities.
    world: hecs::World,

    /// Read events from the systems
    rdr_id: ReaderId<GameEvent>,

    /// Clean up the dead entities.
    garbage_collector: GarbageCollector,

    phantom: PhantomData<A>,
}

impl<'a, A> Game<'a, A>
where
    A: InputAction + 'static,
{
    /// Run the game. This is the main loop.
    pub fn run(&mut self) {
        let mut current_time = Instant::now();
        let dt = Duration::from_millis(16);
        let mut back_buffer = self.surface.back_buffer().unwrap();

        'app: loop {
            // 1. Poll the events and update the Input resource
            // ------------------------------------------------
            let mut resize = false;
            self.surface.window.glfw.poll_events();
            {
                let mut input = self.resources.fetch_mut::<Input<A>>().unwrap();
                input.prepare();
                for (_, event) in self.surface.events_rx.try_iter() {
                    match event {
                        WindowEvent::Close
                        | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,
                        WindowEvent::FramebufferSize(_, _) => resize = true,
                        ev => input.process_event(ev),
                    }
                }
            }

            // 2. Update the scene.
            // ------------------------------------------------
            if let Some(scene) = self.scene_stack.current_mut() {
                scene.update(dt, &mut self.world, &self.resources);

                {
                    let chan = self.resources.fetch::<EventChannel<GameEvent>>().unwrap();
                    for ev in chan.read(&mut self.rdr_id) {
                        scene.process_event(*ev);
                    }
                }
            }

            // 3. Clean up dead entities.
            // ------------------------------------------------
            self.garbage_collector
                .collect(&mut self.world, &self.resources);

            // 4. Render to screen
            // ------------------------------------------------
            log::debug!("RENDER");
            self.renderer
                .update(self.surface, &self.world, &self.resources);
            if resize {
                back_buffer = self.surface.back_buffer().unwrap();
            }

            let render =
                self.renderer
                    .render(self.surface, &mut back_buffer, &self.world, &self.resources);
            if render.is_ok() {
                self.surface.window.swap_buffers();
            } else {
                break 'app;
            }

            let now = Instant::now();
            let frame_duration = now - current_time;
            if frame_duration < dt {
                thread::sleep(dt - frame_duration);
            }
            current_time = now;
        }

        info!("Bye bye.");
    }
}
