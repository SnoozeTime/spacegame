use crate::assets::prefab::PrefabManager;
use crate::assets::sprite::SpriteAsset;
use crate::resources::Resources;
use log::debug;
use luminance::context::GraphicsContext;
use luminance_gl::GL33;
use std::collections::hash_map::Keys;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use thiserror::Error;

pub mod prefab;
pub mod sprite;

pub fn create_asset_managers<S>(_surface: &mut S, resources: &mut Resources)
where
    S: GraphicsContext<Backend = GL33> + 'static,
{
    let base_path = std::env::var("ASSET_PATH").unwrap_or("".to_string());

    let sprite_manager: AssetManager<S, SpriteAsset<S>> = AssetManager::from_loader(Box::new(
        sprite::SpriteSyncLoader::new(PathBuf::from(&base_path).join("assets/sprites")),
    ));

    let prefab_loader: PrefabManager<S> = AssetManager::from_loader(Box::new(
        prefab::PrefabSyncLoader::new(PathBuf::from(&base_path).join("assets/prefab")),
    ));

    resources.insert(sprite_manager);
    resources.insert(prefab_loader);
}

pub fn update_asset_managers<S>(surface: &mut S, resources: &Resources)
where
    S: GraphicsContext<Backend = GL33> + 'static,
{
    {
        let mut sprite_manager = resources
            .fetch_mut::<AssetManager<S, SpriteAsset<S>>>()
            .unwrap();
        sprite_manager.upload_all(surface);
    }

    {
        let mut prefab_loader = resources.fetch_mut::<PrefabManager<S>>().unwrap();
        prefab_loader.upload_all(surface);
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Handle(pub String);

#[derive(Debug, Error)]
pub enum AssetError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    ImageError(#[from] image::ImageError),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
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

pub struct AssetManager<S, T: Default>
where
    S: GraphicsContext<Backend = GL33>,
{
    // might want to use a LRU instead...
    store: HashMap<Handle, Asset<T>>,
    loader: Box<dyn Loader<S, T>>,
}

impl<S, T: Default> AssetManager<S, T>
where
    S: GraphicsContext<Backend = GL33>,
{
    pub fn from_loader(loader: Box<dyn Loader<S, T>>) -> Self {
        Self {
            store: HashMap::new(),
            loader,
        }
    }

    pub fn load(&mut self, asset_name: &str) -> Handle {
        let handle = Handle(asset_name.to_owned());
        if self.store.contains_key(&handle) {
            return handle;
        }
        let asset = self.loader.load(asset_name);
        self.store.insert(handle.clone(), asset);
        handle
    }

    pub fn upload_all(&mut self, ctx: &mut S) {
        // once every now and then, check the resources ready to be uploaded by the current thread.
        for asset in self.store.values() {
            let asset = &mut *asset.asset.lock().unwrap();
            if let LoadingStatus::Loaded(ref mut t) = asset {
                // UPLOAD
                self.loader.upload_to_gpu(ctx, t);
            }
            asset.move_to_read();
        }
    }

    pub fn get(&self, handle: &Handle) -> Option<&Asset<T>> {
        self.store.get(handle)
    }

    pub fn get_mut(&mut self, handle: &Handle) -> Option<&mut Asset<T>> {
        self.store.get_mut(handle)
    }

    pub fn is_loaded(&self, handle: &Handle) -> bool {
        self.store
            .get(handle)
            .map(|asset| asset.is_loaded())
            .unwrap_or(false)
    }

    pub fn is_error(&self, handle: &Handle) -> bool {
        self.store
            .get(handle)
            .map(|asset| asset.is_error())
            .unwrap_or(false)
    }

    /// Return the assets that are currently managed
    pub fn keys(&self) -> Keys<Handle, Asset<T>> {
        self.store.keys()
    }
}

pub trait Loader<S, T>
where
    S: GraphicsContext<Backend = GL33>,
{
    /// Get an asset from an handle
    fn load(&mut self, asset_name: &str) -> Asset<T>;

    fn upload_to_gpu(&self, _ctx: &mut S, _inner: &mut T) {}
}
