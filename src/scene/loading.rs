use crate::assets::audio::Audio;
use crate::assets::sprite::SpriteAsset;
use crate::assets::{AssetManager, Handle};
use crate::core::scene::{Scene, SceneResult};
use crate::core::serialization::SerializedEntity;
use crate::render::ui::{Gui, GuiContext};
use crate::resources::Resources;
use core::time::Duration;
use hecs::World;

pub struct LoadingScene<S: Scene> {
    prefabs_to_load: Vec<String>,
    audio_to_load: Vec<String>,
    sprites_to_load: Vec<String>,
    next_scene: Option<S>,
    audio_handles: Vec<Handle>,
    prefab_handles: Vec<Handle>,
    sprite_handles: Vec<Handle>,
    loaded: usize,
}

impl<S> LoadingScene<S>
where
    S: Scene + 'static,
{
    pub fn new(
        prefabs_to_load: Vec<String>,
        audio_to_load: Vec<String>,
        sprites_to_load: Vec<String>,
        next_scene: S,
    ) -> Self {
        Self {
            prefabs_to_load,
            audio_to_load,
            sprites_to_load,
            next_scene: Some(next_scene),
            prefab_handles: vec![],
            audio_handles: vec![],
            sprite_handles: vec![],
            loaded: 0,
        }
    }
}

impl<S> Scene for LoadingScene<S>
where
    S: Scene + 'static,
{
    fn on_create(&mut self, _world: &mut World, resources: &mut Resources) {
        // Pre-load :)
        let mut prefab_manager = resources
            .fetch_mut::<AssetManager<SerializedEntity>>()
            .unwrap();
        self.prefab_handles = self
            .prefabs_to_load
            .iter()
            .map(|name| prefab_manager.load(name.clone()))
            .collect();

        let mut audio_manager = resources.fetch_mut::<AssetManager<Audio>>().unwrap();
        self.audio_handles = self
            .audio_to_load
            .iter()
            .map(|name| audio_manager.load(name.clone()))
            .collect();
        let mut sprite_manager = resources.fetch_mut::<AssetManager<SpriteAsset>>().unwrap();
        self.sprite_handles = self
            .sprites_to_load
            .iter()
            .map(|name| sprite_manager.load(name.clone()))
            .collect();
    }

    fn update(&mut self, _dt: Duration, _world: &mut World, resources: &Resources) -> SceneResult {
        let prefab_manager = resources
            .fetch_mut::<AssetManager<SerializedEntity>>()
            .unwrap();
        let sprite_manager = resources.fetch_mut::<AssetManager<SpriteAsset>>().unwrap();
        let audio_manager = resources.fetch::<AssetManager<Audio>>().unwrap();
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

        nb_loaded += self
            .sprite_handles
            .iter()
            .filter(|h| {
                info!("{:?} is loaded = {}", h, sprite_manager.is_loaded(h));
                sprite_manager.is_loaded(h)
            })
            .count();
        nb_error += self
            .sprite_handles
            .iter()
            .filter(|h| sprite_manager.is_error(h))
            .count();

        self.loaded = nb_loaded;

        info!(
            "LOADED {} ERROR {} TOTAL {}",
            nb_loaded,
            nb_error,
            self.prefab_handles.len() + self.audio_handles.len() + self.sprite_handles.len()
        );

        if nb_error > 0 {
            // NG
            SceneResult::Pop
        } else if nb_loaded
            == self.prefab_handles.len() + self.audio_handles.len() + self.sprite_handles.len()
        {
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
        let w = gui_context.window_dim.width as f32;
        let h = gui_context.window_dim.height as f32;
        // panel should take up 20% of the window width and 80% of the window height
        let panel_width = w * 0.2;
        let panel_height = h * 0.6;

        let anchor = glam::vec2(w / 2.0 - panel_width / 2.0, h / 2.0 - panel_height / 2.0);

        let mut gui = gui_context.new_frame();
        gui.label(
            anchor,
            format!(
                "{}/{}",
                self.loaded,
                self.prefab_handles.len() + self.audio_handles.len() + self.sprite_handles.len()
            ),
        );
        Some(gui)
    }
}
