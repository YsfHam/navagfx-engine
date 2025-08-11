use std::{any::{Any, TypeId}, collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData, sync::{Arc, Mutex}};


pub mod texture;

pub trait Asset {}

pub type AssetsManagerRef = Arc<Mutex<AssetsManager>>;

pub struct AssetsManager {
    storages: HashMap<TypeId, Box<dyn Any>>
}


impl AssetsManager {
    pub fn new() -> Self {
        Self {
            storages: HashMap::new()
        }
    }

    pub fn register_assets_type<TAsset: 'static>(mut self) -> Self {
        let asset_type_id = TypeId::of::<TAsset>();

        let old_storage = self.storages.insert(asset_type_id, Box::new(AssetsStorage::<TAsset>::new()));

        if old_storage.is_some() {
            panic!("Storage for type {} is already created", std::any::type_name::<TAsset>());
        }

        self
    }

    pub fn store_asset<TAsset: 'static>(&mut self, asset: TAsset) -> AssetHandle<TAsset> {
        self.get_storage_mut().store_asset(asset)
    }

    pub fn get_asset<TAsset: 'static>(&self, handle: AssetHandle<TAsset>) -> &TAsset {
        self.get_storage().get_asset(handle)
    }



    fn get_storage_mut<TAsset: 'static>(&mut self) -> &mut AssetsStorage<TAsset> {
        self.storages.get_mut(&TypeId::of::<TAsset>())
        .and_then(|s| s.downcast_mut::<AssetsStorage<TAsset>>())
        .expect(&format!("No storage created for type {}", std::any::type_name::<TAsset>()))
    }

    fn get_storage<TAsset: 'static>(&self) -> &AssetsStorage<TAsset> {
        self.storages.get(&TypeId::of::<TAsset>())
        .and_then(|s| s.downcast_ref::<AssetsStorage<TAsset>>())
        .expect(&format!("No storage created for type {}", std::any::type_name::<TAsset>()))
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