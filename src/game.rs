#[cfg(feature = "hot-reload")]
use crate::assets::HotReloader;
use crate::config::AudioConfig;
use crate::core::audio::AudioSystem;
use crate::core::camera::{Camera, ProjectionMatrix};
use crate::core::input::ser::{InputEvent, VirtualButton, VirtualKey};
use crate::core::input::{Input, InputAction};
use crate::core::random::{RandomGenerator, Seed};
use crate::core::scene::{Scene, SceneStack};
use crate::core::transform::update_transforms;
use crate::core::window::WindowDim;
use crate::event::GameEvent;
use crate::gameplay::collision::CollisionWorld;
use crate::gameplay::delete::GarbageCollector;
use crate::render::path::debug::DebugQueue;
use crate::render::ui::gui::GuiContext;
use crate::render::{Context, Renderer};
use crate::resources::Resources;
use crate::{HEIGHT, WIDTH};
use log::info;
use luminance_front::framebuffer::Framebuffer;
use luminance_front::texture::Dim2;
use shrev::{EventChannel, ReaderId};
use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::thread;
use std::time::{Duration, Instant};

/// GameBuilder is used to create a new game. Game struct has a lot of members that do not need to be
/// exposed so gamebuilder provides a simpler way to get started.
pub struct GameBuilder<A>
where
    A: InputAction,
{
    scene: Option<Box<dyn Scene>>,
    resources: Resources,
    phantom: PhantomData<A>,
    seed: Option<Seed>,
    input_config: Option<(HashMap<VirtualKey, A>, HashMap<VirtualButton, A>)>,
    gui_context: GuiContext,
    audio_config: AudioConfig,
}

impl<A> GameBuilder<A>
where
    A: InputAction + 'static,
{
    pub fn new() -> Self {
        // resources will need at least an event channel and an input
        let mut resources = Resources::default();
        let chan: EventChannel<GameEvent> = EventChannel::new();
        resources.insert(chan);

        // the proj matrix.
        resources.insert(ProjectionMatrix::new(WIDTH as f32, HEIGHT as f32));
        resources.insert(WindowDim::new(WIDTH, HEIGHT));
        resources.insert(CollisionWorld::default());
        resources.insert(DebugQueue::default());

        Self {
            gui_context: GuiContext::new(WindowDim::new(WIDTH, HEIGHT)),
            scene: None,
            resources,
            input_config: None,
            phantom: PhantomData::default(),
            seed: None,
            audio_config: AudioConfig::default(),
        }
    }

    /// Set up the first scene.
    pub fn for_scene(mut self, scene: Box<dyn Scene>) -> Self {
        self.scene = Some(scene);
        self
    }

    pub fn with_input_config(
        mut self,
        key_map: HashMap<VirtualKey, A>,
        btn_map: HashMap<VirtualButton, A>,
    ) -> Self {
        self.input_config = Some((key_map, btn_map));
        self
    }

    /// Specific config for audio
    pub fn with_audio_config(mut self, audio_config: AudioConfig) -> Self {
        self.audio_config = audio_config;
        self
    }

    /// Add custom resources.
    pub fn with_resource<T: Any>(mut self, r: T) -> Self {
        self.resources.insert(r);
        self
    }

    pub fn with_seed(mut self, seed: Seed) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn build(mut self, surface: &mut Context) -> Game<A> {
        info!("Building Renderer");
        let renderer = Renderer::new(surface, &self.gui_context);

        // and some asset manager;
        info!("Creating asset managers");
        crate::assets::create_asset_managers(surface, &mut self.resources);

        // Need some input :D
        info!("Mapping inputs");
        let input: Input<A> = {
            let (key_mapping, btn_mapping) = self
                .input_config
                .unwrap_or((A::get_default_key_mapping(), A::get_default_mouse_mapping()));
            Input::new(key_mapping, btn_mapping)
        };
        self.resources.insert(input);

        info!("Creating world");
        let mut world = hecs::World::new();

        info!("Random seed");
        // if a seed is provided, let's add it to the resources.
        if let Some(seed) = self.seed {
            self.resources.insert(RandomGenerator::new(seed));
        } else {
            self.resources.insert(RandomGenerator::from_entropy());
        }

        info!("Creating scene stack");
        let scene_stack = {
            let mut scenes = SceneStack::default();
            if let Some(scene) = self.scene {
                scenes.push(scene, &mut world, &mut self.resources);
            }
            scenes
        };

        info!("Setting up reader from event channel");
        let rdr_id = {
            let mut chan = self
                .resources
                .fetch_mut::<EventChannel<GameEvent>>()
                .unwrap();
            chan.register_reader()
        };
        info!("Creating garbage collector");
        let garbage_collector = GarbageCollector::new(&mut self.resources);

        // we need a camera :)
        info!("Creating camera");
        world.spawn((Camera::new(),));

        // audio system.
        info!("Creating audio system");
        let audio_system = if cfg!(target_arch = "wasm32") {
            None
        } else {
            Some(
                AudioSystem::new(&self.resources, self.audio_config)
                    .expect("Cannot create audio system"),
            )
        };

        info!("Finished building game");

        Game {
            renderer,
            scene_stack,
            world,
            audio_system,
            audio_config: self.audio_config,
            resources: self.resources,
            rdr_id,
            garbage_collector,
            phantom: self.phantom,
            gui_context: self.gui_context,
            #[cfg(feature = "hot-reload")]
            hot_reloader: HotReloader::new(),
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
pub struct Game<A> {
    /// for drawing stuff
    renderer: Renderer,

    /// All the scenes. Current scene will be used in the main loop.
    scene_stack: SceneStack,

    /// Play music and sound effects
    audio_config: AudioConfig,
    audio_system: Option<AudioSystem>,

    /// Resources (assets, inputs...)
    pub(crate) resources: Resources,

    /// Current entities.
    world: hecs::World,

    /// Read events from the systems
    rdr_id: ReaderId<GameEvent>,

    /// Clean up the dead entities.
    garbage_collector: GarbageCollector,

    gui_context: GuiContext,

    phantom: PhantomData<A>,

    #[cfg(feature = "hot-reload")]
    hot_reloader: HotReloader,
}

impl<A> Game<A>
where
    A: InputAction + 'static,
{
    /// In case of wasm, the audio system must be created after user interaction (auto play policy
    /// of browsers)
    pub fn create_audio_system(&mut self) {
        if self.audio_system.is_none() {
            self.audio_system = Some(
                AudioSystem::new(&self.resources, self.audio_config)
                    .expect("Cannot create audio system"),
            );
        }
    }

    /// Run the game. This is the main loop.
    pub fn run(&mut self, surface: &mut Context) {
        let mut current_time = Instant::now();
        let dt = Duration::from_millis(16);
        let mut back_buffer = surface.back_buffer().unwrap();

        'app: loop {
            self.prepare_input();

            let should_continue = self.run_frame(surface, &mut back_buffer, dt);

            if !should_continue {
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

    pub fn prepare_input(&mut self) {
        let mut input = self.resources.fetch_mut::<Input<A>>().unwrap();
        input.prepare();
        self.gui_context.reset_inputs();
    }

    pub fn process_input(&mut self, input_event: InputEvent) {
        let mut input = self.resources.fetch_mut::<Input<A>>().unwrap();

        self.gui_context.process_event(input_event.clone());
        if let Some(scene) = self.scene_stack.current_mut() {
            scene.process_input(&mut self.world, input_event.clone(), &self.resources);
        }
        input.process_event(input_event)
    }

    pub fn run_frame(
        &mut self,
        surface: &mut Context,
        mut back_buffer: &mut Framebuffer<Dim2, (), ()>,
        dt: Duration,
    ) -> bool {
        let mut resize = false;
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            use glfw::WindowEvent;
            surface.window.glfw.poll_events();
            {
                for (_, event) in surface.events_rx.try_iter() {
                    match event {
                        WindowEvent::Close => return false,
                        WindowEvent::FramebufferSize(_, _) => resize = true,
                        ev => {
                            let ev: InputEvent = ev.into();
                            self.process_input(ev);
                        }
                    }
                }
            }
        }

        // 2. Update the scene.
        // ------------------------------------------------
        trace!("Update scene");

        let scene_result = if let Some(scene) = self.scene_stack.current_mut() {
            let scene_res = scene.update(dt, &mut self.world, &self.resources);

            {
                let chan = self.resources.fetch::<EventChannel<GameEvent>>().unwrap();
                for ev in chan.read(&mut self.rdr_id) {
                    scene.process_event(&mut self.world, ev.clone(), &self.resources);
                }
            }

            let maybe_gui =
                scene.prepare_gui(dt, &mut self.world, &self.resources, &mut self.gui_context);

            self.renderer.prepare_ui(
                surface,
                maybe_gui,
                &self.resources,
                &mut *self.gui_context.fonts.borrow_mut(),
            );

            Some(scene_res)
        } else {
            None
        };

        // Update children transforms:
        // -----------------------------
        update_transforms(&mut self.world);

        // 3. Clean up dead entities.
        // ------------------------------------------------
        self.garbage_collector
            .collect(&mut self.world, &self.resources);

        // 4. Render to screen
        // ------------------------------------------------
        self.renderer
            .update(surface, &self.world, dt, &self.resources);
        if resize {
            *back_buffer = surface.back_buffer().unwrap();
            let new_size = back_buffer.size();
            let mut proj = self.resources.fetch_mut::<ProjectionMatrix>().unwrap();
            proj.resize(new_size[0] as f32, new_size[1] as f32);

            let mut dim = self.resources.fetch_mut::<WindowDim>().unwrap();
            dim.resize(new_size[0], new_size[1]);
            self.gui_context.window_dim = *dim;
        }

        trace!("Render");
        let render = self
            .renderer
            .render(surface, &mut back_buffer, &self.world, &self.resources);
        if render.is_ok() {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            {
                use glfw::Context;
                surface.window.swap_buffers();
            }
        } else {
            return false;
        }

        // Play music :)
        trace!("Process audio system");
        if let Some(ref mut audio) = self.audio_system {
            audio.process(&self.resources);
        }

        // Update collision world for collision queries.
        {
            let mut collisions = self.resources.fetch_mut::<CollisionWorld>().unwrap();
            collisions.synchronize(&self.world);
        }

        // Either clean up or load new resources.
        crate::assets::update_asset_managers(surface, &self.resources);
        #[cfg(feature = "hot-reload")]
        self.hot_reloader.update(&self.resources);

        // Now, if need to switch scenes, do it.
        if let Some(res) = scene_result {
            self.scene_stack
                .apply_result(res, &mut self.world, &mut self.resources);
        }

        true
    }
}
