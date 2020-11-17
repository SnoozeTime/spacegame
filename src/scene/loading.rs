use crate::assets::prefab::PrefabManager;
use crate::assets::Handle;
use crate::core::scene::{Scene, SceneResult};
use crate::resources::Resources;
use crate::scene::MainScene;
use bitflags::_core::time::Duration;
use hecs::World;
use luminance_glfw::GlfwSurface;

pub struct LoadingScene {
    prefabs_to_load: Vec<String>,
    handles: Vec<Handle>,
}

impl LoadingScene {
    pub fn new(prefabs_to_load: Vec<String>) -> Self {
        Self {
            prefabs_to_load,
            handles: vec![],
        }
    }
}

impl Scene for LoadingScene {
    fn on_create(&mut self, _world: &mut World, resources: &mut Resources) {
        // Pre-load :)
        let mut prefab_manager = resources.fetch_mut::<PrefabManager<GlfwSurface>>().unwrap();

        self.handles = self
            .prefabs_to_load
            .iter()
            .map(|name| prefab_manager.load(name.as_str()))
            .collect();
    }

    fn update(&mut self, _dt: Duration, _world: &mut World, resources: &Resources) -> SceneResult {
        let prefab_manager = resources.fetch_mut::<PrefabManager<GlfwSurface>>().unwrap();

        // loaded.
        let nb_loaded = self
            .handles
            .iter()
            .filter(|h| prefab_manager.is_loaded(h))
            .count();
        let nb_error = self
            .handles
            .iter()
            .filter(|h| prefab_manager.is_error(h))
            .count();

        if nb_error > 0 {
            // NG
            SceneResult::Pop
        } else if nb_loaded == self.handles.len() {
            SceneResult::ReplaceScene(Box::new(MainScene::new()))
        } else {
            SceneResult::Noop
        }
    }
}
