use crate::assets::audio::Audio;
use crate::assets::prefab::PrefabManager;
use crate::assets::{AssetManager, Handle};
use crate::core::scene::{Scene, SceneResult};
use crate::resources::Resources;
use crate::scene::MainScene;
use bitflags::_core::time::Duration;
use hecs::World;
use luminance_glfw::GlfwSurface;

pub struct LoadingScene<S: Scene> {
    prefabs_to_load: Vec<String>,
    audio_to_load: Vec<String>,
    next_scene: Option<S>,
    audio_handles: Vec<Handle>,
    prefab_handles: Vec<Handle>,
}

impl<S> LoadingScene<S>
where
    S: Scene + 'static,
{
    pub fn new(prefabs_to_load: Vec<String>, audio_to_load: Vec<String>, next_scene: S) -> Self {
        Self {
            prefabs_to_load,
            audio_to_load,
            next_scene: Some(next_scene),
            prefab_handles: vec![],
            audio_handles: vec![],
        }
    }
}

impl<S> Scene for LoadingScene<S>
where
    S: Scene + 'static,
{
    fn on_create(&mut self, _world: &mut World, resources: &mut Resources) {
        // Pre-load :)
        let mut prefab_manager = resources.fetch_mut::<PrefabManager<GlfwSurface>>().unwrap();
        self.prefab_handles = self
            .prefabs_to_load
            .iter()
            .map(|name| prefab_manager.load(name.clone()))
            .collect();

        let mut audio_manager = resources
            .fetch_mut::<AssetManager<GlfwSurface, Audio>>()
            .unwrap();
        self.audio_handles = self
            .audio_to_load
            .iter()
            .map(|name| audio_manager.load(name.clone()))
            .collect();
    }

    fn update(&mut self, _dt: Duration, _world: &mut World, resources: &Resources) -> SceneResult {
        let prefab_manager = resources.fetch_mut::<PrefabManager<GlfwSurface>>().unwrap();
        let mut audio_manager = resources
            .fetch_mut::<AssetManager<GlfwSurface, Audio>>()
            .unwrap();
        // loaded.
        let mut nb_loaded = self
            .prefab_handles
            .iter()
            .filter(|h| prefab_manager.is_loaded(h))
            .count();
        let mut nb_error = self
            .prefab_handles
            .iter()
            .filter(|h| prefab_manager.is_error(h))
            .count();

        nb_loaded += self
            .audio_handles
            .iter()
            .filter(|h| {
                info!("{:?} is loaded = {}", h, audio_manager.is_loaded(h));
                audio_manager.is_loaded(h)
            })
            .count();
        nb_error += self
            .audio_handles
            .iter()
            .filter(|h| audio_manager.is_error(h))
            .count();

        if nb_error > 0 {
            // NG
            SceneResult::Pop
        } else if nb_loaded == self.prefab_handles.len() + self.audio_handles.len() {
            SceneResult::ReplaceScene(Box::new(self.next_scene.take().unwrap()))
        } else {
            SceneResult::Noop
        }
    }
}
