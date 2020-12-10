use crate::assets::audio::Audio;
use crate::assets::shader::ShaderManager;
use crate::assets::sprite::SpriteAsset;
use crate::core::serialization::SerializedEntity;
use crate::paths::get_assets_path;
use crate::render::Context;
use crate::resources::Resources;
use log::debug;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::hash_map::Keys;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use thiserror::Error;

// #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), not(feature = "web")))]
pub mod audio;

pub mod json;
pub mod prefab;
pub mod shader;
pub mod sprite;

#[cfg(target_arch = "wasm32")]
pub fn create_asset_managers(_surface: &mut Context, resources: &mut Resources) {
    let base_path = get_assets_path();

    let sprite_manager: AssetManager<SpriteAsset> =
        AssetManager::from_loader(Box::new(sprite::AsyncWebLoader));

    let audio_loader: AssetManager<Audio> =
        AssetManager::from_loader(Box::new(audio::AsyncWebLoader::new("/assets/".to_string())));
    resources.insert(audio_loader);

    let serialized_entity_loader: AssetManager<SerializedEntity> = AssetManager::from_loader(
        Box::new(json::AsyncWebLoader::new("/assets/serialized/".to_string())),
    );
    resources.insert(serialized_entity_loader);

    let shader_loader: ShaderManager = AssetManager::from_loader(Box::new(
        shader::ShaderLoader::new(base_path.join("shaders")),
    ));
    resources.insert(sprite_manager);
    resources.insert(shader_loader);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_asset_managers(_surface: &mut Context, resources: &mut Resources) {
    let base_path = get_assets_path();

    #[cfg(not(feature = "packed"))]
    let sprite_manager: AssetManager<SpriteAsset> = AssetManager::from_loader(Box::new(
        sprite::SpriteSyncLoader::new(base_path.join("sprites")),
    ));

    #[cfg(feature = "packed")]
    let sprite_manager: AssetManager<SpriteAsset> = AssetManager::from_loader(Box::new(
        sprite::SpritePackLoader::new(base_path.join("sprites")),
    ));

    let serialized_entity_loader: AssetManager<SerializedEntity> = AssetManager::from_loader(
        Box::new(json::JsonSyncLoader::new(base_path.join("serialized"))),
    );
    resources.insert(serialized_entity_loader);

    let audio_loader: AssetManager<Audio> =
        AssetManager::from_loader(Box::new(audio::AudioSyncLoader::new(base_path.clone())));
    resources.insert(audio_loader);

    let shader_loader: ShaderManager = AssetManager::from_loader(Box::new(
        shader::ShaderLoader::new(base_path.join("shaders")),
    ));
    resources.insert(sprite_manager);
    resources.insert(prefab_loader);
    resources.insert(shader_loader);
}

pub fn update_asset_managers(surface: &mut Context, resources: &Resources) {
    {
        let mut sprite_manager = resources.fetch_mut::<AssetManager<SpriteAsset>>().unwrap();
        sprite_manager.upload_all(surface);
    }

    {
        let mut audio_loader = resources.fetch_mut::<AssetManager<Audio>>().unwrap();
        audio_loader.upload_all(surface);
    }

    {
        let mut shader_loader = resources.fetch_mut::<ShaderManager>().unwrap();
        shader_loader.upload_all(surface);
    }
    {
        let mut serialized_manager = resources
            .fetch_mut::<AssetManager<SerializedEntity>>()
            .unwrap();
        serialized_manager.upload_all(surface);
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Handle<H = String>(pub H);

#[derive(Debug, Error)]
pub enum AssetError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    ImageError(#[from] image::ImageError),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    #[error(transparent)]
    ShaderError(#[from] luminance::shader::ProgramError),

    #[error(transparent)]
    TextureError(#[from] luminance::texture::TextureError),

    #[error("Cannot find {0} in packed data")]
    PackedError(String),
}

pub struct Asset<T> {
    asset: Arc<Mutex<LoadingStatus<T, AssetError>>>,
}

impl<T> Clone for Asset<T> {
    fn clone(&self) -> Self {
        Asset {
            asset: Arc::clone(&self.asset),
        }
    }
}

impl<T> Default for Asset<T> {
    fn default() -> Self {
        Asset::new()
    }
}

impl<T> From<AssetError> for Asset<T> {
    fn from(e: AssetError) -> Self {
        Self {
            asset: Arc::new(Mutex::new(LoadingStatus::Error(e))),
        }
    }
}

impl<T> Asset<T> {
    pub fn new() -> Self {
        Self {
            asset: Arc::new(Mutex::new(LoadingStatus::Loading)),
        }
    }

    pub fn from_asset(asset: T) -> Self {
        Self {
            asset: Arc::new(Mutex::new(LoadingStatus::Ready(asset))),
        }
    }

    pub fn set_ready(&mut self, v: T) {
        *self.asset.lock().unwrap() = LoadingStatus::Loaded(v);
    }

    pub fn set_loaded(&mut self, v: T) {
        *self.asset.lock().unwrap() = LoadingStatus::Loaded(v);
    }

    pub fn set_error(&mut self, e: AssetError) {
        *self.asset.lock().unwrap() = LoadingStatus::Error(e);
    }

    /// Returns true if the asset has finished loading.
    pub fn is_loaded(&self) -> bool {
        let asset = &*self.asset.lock().unwrap();
        if let LoadingStatus::Ready(_) = asset {
            true
        } else {
            false
        }
    }

    /// Returns true if the asset has failed loading.
    pub fn is_error(&self) -> bool {
        let asset = &*self.asset.lock().unwrap();
        if let LoadingStatus::Error(_) = asset {
            true
        } else {
            false
        }
    }

    /// Execute a function only if the asset is loaded.
    pub fn execute<F, Ret>(&self, mut f: F) -> Option<Ret>
    where
        F: FnMut(&T) -> Ret,
    {
        let asset = &*self.asset.lock().unwrap();
        if let LoadingStatus::Ready(ref inner) = asset {
            Some(f(inner))
        } else {
            None
        }
    }

    /// Execute a function only if the asset is loaded.
    pub fn execute_mut<F, Ret>(&self, mut f: F) -> Option<Ret>
    where
        F: FnMut(&mut T) -> Ret,
    {
        let asset = &mut *self.asset.lock().unwrap();
        if let LoadingStatus::Ready(ref mut inner) = asset {
            debug!("Asset is ready");
            Some(f(inner))
        } else {
            debug!("Asset is not ready");
            None
        }
    }
}
impl<T: Clone> Asset<T> {
    /// Some assets should not be modified so it's better to get a copy of them
    /// (Dialog for example)
    pub fn clone_inner(&self) -> Option<T> {
        let asset = &*self.asset.lock().unwrap();
        if let LoadingStatus::Loaded(ref inner) = asset {
            Some((*inner).clone())
        } else {
            None
        }
    }
}

pub enum LoadingStatus<T, E> {
    Ready(T),
    Loaded(T),
    Loading,
    Error(E),
}

impl<T: Default, E> LoadingStatus<T, E> {
    pub fn move_to_read(&mut self) {
        match self {
            LoadingStatus::Loaded(asset) => *self = LoadingStatus::Ready(std::mem::take(asset)),
            _ => (),
        }
    }
}

pub struct AssetManager<T: Default, H = String>
where
    H: Clone,
{
    // might want to use a LRU instead...
    store: HashMap<Handle<H>, Asset<T>>,
    loader: Box<dyn Loader<T, H>>,
}

impl<T: Default, H> AssetManager<T, H>
where
    H: Clone + Eq + PartialEq + Hash,
{
    pub fn from_loader(loader: Box<dyn Loader<T, H>>) -> Self {
        Self {
            store: HashMap::new(),
            loader,
        }
    }

    pub fn load(&mut self, asset_name: H) -> Handle<H> {
        let handle = Handle(asset_name.clone());
        if self.store.contains_key(&handle) {
            return handle;
        }
        let asset = self.loader.load(asset_name);
        self.store.insert(handle.clone(), asset);
        handle
    }

    pub fn reload(&mut self, asset_name: H) -> Handle<H> {
        let handle = Handle(asset_name.clone());
        let asset = self.loader.load(asset_name);
        self.store.insert(handle.clone(), asset);
        handle
    }

    pub fn upload_all(&mut self, ctx: &mut Context) {
        // once every now and then, check the resources ready to be uploaded by the current thread.
        for asset in self.store.values() {
            let asset = &mut *asset.asset.lock().unwrap();

            let mut has_error = Ok(());
            let mut to_process = false;
            if let LoadingStatus::Loaded(ref mut t) = asset {
                to_process = true;
                // UPLOAD
                has_error = self.loader.upload_to_gpu(ctx, t);
            }

            if to_process {
                if let Err(e) = has_error {
                    error!("Error when uploading to GPU = {:?}", e);
                    *asset = LoadingStatus::Error(e);
                } else {
                    asset.move_to_read();
                }
            }
        }
    }

    pub fn get(&self, handle: &Handle<H>) -> Option<&Asset<T>> {
        self.store.get(handle)
    }

    pub fn get_mut(&mut self, handle: &Handle<H>) -> Option<&mut Asset<T>> {
        self.store.get_mut(handle)
    }

    pub fn is_loaded(&self, handle: &Handle<H>) -> bool {
        self.store
            .get(handle)
            .map(|asset| asset.is_loaded())
            .unwrap_or(false)
    }

    pub fn is_error(&self, handle: &Handle<H>) -> bool {
        self.store
            .get(handle)
            .map(|asset| asset.is_error())
            .unwrap_or(false)
    }

    /// Return the assets that are currently managed
    pub fn keys(&self) -> Keys<Handle<H>, Asset<T>> {
        self.store.keys()
    }
}

pub trait Loader<T, H = String>
where
    H: Clone,
{
    /// Get an asset from an handle
    fn load(&mut self, asset_name: H) -> Asset<T>;

    fn upload_to_gpu(&self, _ctx: &mut Context, _inner: &mut T) -> Result<(), AssetError> {
        Ok(())
    }
}

/// Good for development. Will listen to the asset folder and ask the asset managers to reload their
/// data if needed
#[cfg(feature = "hot-reload")]
pub struct HotReloader {
    rx: Receiver<Result<notify::Event, notify::Error>>,
    _watcher: RecommendedWatcher,
}

#[cfg(feature = "hot-reload")]
impl HotReloader {
    pub fn new() -> Self {
        let base_path = get_assets_path();

        let (tx, rx) = std::sync::mpsc::channel();

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.

        // Automatically select the best implementation for your platform.
        // You can also access each implementation directly e.g. INotifyWatcher.
        let mut watcher: RecommendedWatcher =
            Watcher::new_immediate(move |res| tx.send(res).unwrap()).unwrap();

        watcher
            .watch(base_path.clone(), RecursiveMode::Recursive)
            .unwrap();
        Self {
            rx,
            _watcher: watcher,
        }
    }

    /// Will check if there is a file that has changed and will reload the corresponding resource.
    ///
    /// WIP, currently just reload all the shaders :D
    pub fn update(&mut self, resources: &Resources) {
        let mut should_reload = false;
        for res in &self.rx.try_recv() {
            match res {
                Ok(Event {
                    kind: EventKind::Modify(..),
                    paths,
                    ..
                }) => {
                    debug!("Should reload {:?}", paths);
                    should_reload = true
                }
                _ => (),
            }
        }

        if should_reload {
            if let Some(mut shader_manager) = resources.fetch_mut::<ShaderManager>() {
                let keys = { shader_manager.keys().map(|k| k.clone()).collect::<Vec<_>>() };
                for k in keys {
                    shader_manager.reload(k.0);
                }
            }
        }
    }
}
