use std::{any::{Any, TypeId}, collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData, sync::{Arc, Mutex}};


pub mod texture;
pub mod loaders;

#[derive(Debug)]
pub enum AssetsManagerError<E> {
    AssetAlreadyRegistered(&'static str),
    UnregisteredAsset(&'static str),
    UnregisteredLoader(&'static str),
    LoadingError(E)
}

impl<E> From<E> for AssetsManagerError<E> {
    fn from(error: E) -> Self {
        Self::LoadingError(error)
    }
}

pub type Result<T, E = ()> = std::result::Result<T, AssetsManagerError<E>>;

pub trait Asset {}
pub trait AssetHasDefaultLoader<Src> {
    type Loader: AssetsLoader<Src, TAsset = Self>;
}

pub type AssetsManagerRef = Arc<Mutex<AssetsManager>>;

pub trait AssetsLoader<Src> {
    type TAsset: Asset;
    type Error;

    fn load(&self, source: Src) -> std::result::Result<Self::TAsset, Self::Error>;
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
struct AssetTypeId(TypeId);

impl AssetTypeId {
    fn of<TAsset: Asset + 'static>() -> Self {
        Self(TypeId::of::<TAsset>())
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
struct LoaderTypeId(TypeId);

impl LoaderTypeId {
    fn of<Loader: AssetsLoader<Src> + 'static, Src>() -> Self {
        Self(TypeId::of::<Loader>())
    }
}

pub struct AssetsManager {
    storages: HashMap<AssetTypeId, Box<dyn Any>>,
    loaders: HashMap<LoaderTypeId, Box<dyn Any>>
}


impl AssetsManager {
    pub fn new() -> Self {
        Self {
            storages: HashMap::new(),
            loaders: HashMap::new()
        }
    }

    pub fn register_assets_type<TAsset: 'static + Asset>(&mut self) -> Result<()> {
        let asset_type_id = AssetTypeId::of::<TAsset>();

        let old_stroage = self
            .storages
            .insert(asset_type_id, Box::new(AssetsStorage::<TAsset>::new()));

        match old_stroage {
            Some(_) => Err(AssetsManagerError::AssetAlreadyRegistered(std::any::type_name::<TAsset>())),
            None => Ok(())
        }
    }

    pub fn store_asset<TAsset: 'static + Asset>(&mut self, asset: TAsset) -> Result<AssetHandle<TAsset>> {
        self.get_storage_mut()
        .map(|storage| storage.store_asset(asset))
    }

    pub fn get_asset<TAsset: 'static + Asset>(&self, handle: AssetHandle<TAsset>) -> &TAsset {
        self.get_storage()
        .map(|storage| storage.get_asset(handle))
        .unwrap()
    }

    pub fn register_loader<Loader, TAsset, Src>(&mut self, loader: Loader)
    where 
        Loader: AssetsLoader<Src, TAsset = TAsset> + 'static,
        TAsset: Asset + 'static
    {
        let loader_type_id = LoaderTypeId::of::<Loader, Src>();
        self.loaders.insert(loader_type_id, Box::new(loader));
    }

    pub fn load_asset<TAsset, Src>(&mut self, source: Src) -> 
    Result<
        AssetHandle<TAsset>,
        <TAsset::Loader as AssetsLoader<Src>>::Error
    >
    where
        TAsset: Asset + AssetHasDefaultLoader<Src> + 'static,
        TAsset::Loader: 'static
    {
        self.load_asset_with::<TAsset, TAsset::Loader, Src>(source)
    }

    pub fn load_asset_with<TAsset: 'static + Asset, Loader, Src>(&mut self, source: Src) -> 
        Result<AssetHandle<TAsset>, Loader::Error> 
    where
        Loader: AssetsLoader<Src, TAsset = TAsset> + 'static
    {
        let loader = self.get_loader::<TAsset, Loader, Src>()?;
        let asset = loader.load(source)?;


        self.get_storage_mut()
        .map(|storage| storage.store_asset(asset)   )
    }


    fn get_storage_mut<TAsset: 'static + Asset, E>(&mut self) -> Result<&mut AssetsStorage<TAsset>, E> {
        self.storages.get_mut(&AssetTypeId::of::<TAsset>())
        .and_then(|s| s.downcast_mut::<AssetsStorage<TAsset>>())
        .ok_or(AssetsManagerError::UnregisteredAsset(std::any::type_name::<TAsset>()))
    }

    fn get_storage<TAsset: 'static + Asset>(&self) -> Result<&AssetsStorage<TAsset>> {
        self.storages.get(&AssetTypeId::of::<TAsset>())
        .and_then(|s| s.downcast_ref::<AssetsStorage<TAsset>>())
        .ok_or(AssetsManagerError::UnregisteredAsset(std::any::type_name::<TAsset>()))
    }

    fn get_loader<TAsset: 'static + Asset, Loader, Src>(&self) -> Result<&Loader, Loader::Error>
    where
        Loader: AssetsLoader<Src, TAsset = TAsset> + 'static
    {
        self.loaders.get(&LoaderTypeId::of::<Loader, Src>())
        .and_then(|l| l.downcast_ref::<Loader>())
        .ok_or(AssetsManagerError::UnregisteredLoader(std::any::type_name::<Loader>()))
    }

}


pub struct AssetHandle<T> {
    id: u32,
    _marker: PhantomData<T>
}

impl<T> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self { id: self.id.clone(), _marker: self._marker.clone() }
    }
}
impl<T> Copy for AssetHandle<T> {}

impl<T> Debug for AssetHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetHandle").field("id", &self.id).field("_marker", &self._marker).finish()
    }
}

impl<T> Hash for AssetHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self._marker.hash(state);
    }
}

impl<T> PartialEq for AssetHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self._marker == other._marker
    }
}

impl<T> Eq for AssetHandle<T> {}

impl<T> AssetHandle<T> {
    fn new(id: u32) -> Self {
        Self {
            id,
            _marker: PhantomData
        }
    }
}

struct AssetsStorage<T> {
    next_id: u32,
    storage: HashMap<u32, T>,
}

impl<T> AssetsStorage<T> {
    fn new() -> Self {
        Self {
            next_id: 0,
            storage: HashMap::new(),
        }
    }

    fn store_asset(&mut self, asset: T) -> AssetHandle<T> {
        let handle = self.next_id;

        self.storage.insert(handle, asset);
        self.next_id += 1;

        AssetHandle::new(handle)
    }

    fn get_asset(&self, handle: AssetHandle<T>) -> &T {
        self.storage.get(&handle.id).unwrap()
    }
}