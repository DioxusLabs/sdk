use dioxus::prelude::{use_ref, ScopeState, UseRef};
use postcard::to_allocvec;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::ops::{Deref, DerefMut};

pub fn serde_to_string<T: Serialize>(value: &T) -> String {
    let serialized = to_allocvec(value).unwrap();
    let compressed = yazi::compress(
        &serialized,
        yazi::Format::Zlib,
        yazi::CompressionLevel::BestSize,
    )
    .unwrap();
    let as_str: String = compressed
        .iter()
        .flat_map(|u| {
            [
                char::from_digit(((*u & 0xF0) >> 4).into(), 16).unwrap(),
                char::from_digit((*u & 0x0F).into(), 16).unwrap(),
            ]
            .into_iter()
        })
        .collect();
    as_str
}

pub fn serde_from_string<T: DeserializeOwned>(value: &str) -> T {
    try_serde_from_string(value).unwrap()
}

pub fn try_serde_from_string<T: DeserializeOwned>(value: &str) -> Option<T> {
    let mut bytes: Vec<u8> = Vec::new();
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        let n1 = c.to_digit(16)?;
        let c2 = chars.next()?;
        let n2 = c2.to_digit(16)?;
        bytes.push((n1 * 16 + n2) as u8);
    }
    let (decompressed, _) = yazi::decompress(&bytes, yazi::Format::Zlib).ok()?;
    postcard::from_bytes(&decompressed).ok()
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PersistentStorage {
    pub data: Vec<Vec<u8>>,
    pub idx: usize,
}

pub trait StorageBacking {
    type Key;

    fn get<T: DeserializeOwned>(key: &Self::Key) -> Option<T>;
    fn set<T: Serialize>(key: Self::Key, value: &T);
}

#[derive(Clone, Default)]
pub struct StorageEntry<S: StorageBacking, T: Serialize + DeserializeOwned> {
    key: S::Key,
    pub(crate) data: T,
}

impl<S: StorageBacking, T: Display + Serialize + DeserializeOwned> Display for StorageEntry<S, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

impl<S: StorageBacking, T: Debug + Serialize + DeserializeOwned> Debug for StorageEntry<S, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

impl<S, T> StorageEntry<S, T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned,
    S::Key: Clone,
{
    pub fn new(key: S::Key, data: T) -> Self {
        Self { key, data }
    }

    pub(crate) fn save(&self) {
        S::set(self.key.clone(), &self.data);
    }

    pub fn write(&mut self) -> StorageEntryMut<'_, S, T> {
        StorageEntryMut {
            storage_entry: self,
        }
    }

    pub fn with_mut(&mut self, f: impl FnOnce(&mut T)) {
        f(&mut self.data);
        self.save();
    }
}

impl<S: StorageBacking, T: Serialize + DeserializeOwned> Deref for StorageEntry<S, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub struct StorageEntryMut<'a, S, T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned,
    S::Key: Clone,
{
    storage_entry: &'a mut StorageEntry<S, T>,
}

impl<'a, S, T> Deref for StorageEntryMut<'a, S, T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned,
    S::Key: Clone,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.storage_entry.data
    }
}

impl<'a, S, T> DerefMut for StorageEntryMut<'a, S, T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned,
    S::Key: Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.storage_entry.data
    }
}

impl<'a, S, T> Drop for StorageEntryMut<'a, S, T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned,
    S::Key: Clone,
{
    fn drop(&mut self) {
        self.storage_entry.save();
    }
}

pub fn storage_entry<S: StorageBacking, T: Serialize + DeserializeOwned>(
    key: S::Key,
    init: impl FnOnce() -> T,
) -> T {
    S::get(&key).unwrap_or_else(|| {
        let data = init();
        S::set(key, &data);
        data
    })
}

pub fn synced_storage_entry<S, T>(key: S::Key, init: impl FnOnce() -> T) -> StorageEntry<S, T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned,
    S::Key: Clone,
{
    let data = storage_entry::<S, T>(key.clone(), init);

    StorageEntry::new(key, data)
}

pub fn use_synced_storage_entry<S, T>(
    cx: &ScopeState,
    key: S::Key,
    init: impl FnOnce() -> T,
) -> &UseRef<StorageEntry<S, T>>
where
    S: StorageBacking + 'static,
    T: Serialize + DeserializeOwned + 'static,
    S::Key: Clone,
{
    use_ref(cx, || synced_storage_entry(key, init))
}
