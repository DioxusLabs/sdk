use dioxus::prelude::ScopeState;
use dioxus_signals::{use_signal, Signal};
use postcard::to_allocvec;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

use crate::utils::channel::{use_listen_channel, UseChannel};

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

pub trait StorageBacking: Sized + Clone + 'static {
    type Key: Eq + PartialEq + Hash + Clone + Debug;
    fn subscribe<T: DeserializeOwned + 'static>(
        cx: &ScopeState,
        key: &Self::Key,
    ) -> Option<UseChannel<StorageChannelPayload<Self>>>;
    fn get<T: DeserializeOwned>(key: &Self::Key) -> Option<T>;
    fn set<T: Serialize>(key: Self::Key, value: &T);
}

#[derive(Clone)]
pub struct StorageChannelPayload<S: StorageBacking> {
    pub key: S::Key,
}

impl<S: StorageBacking> Debug for StorageChannelPayload<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StorageChannelPayload")
            .field("key", &self.key)
            .finish()
    }
}

#[derive(Clone, Default)]
pub struct StorageEntry<S: StorageBacking, T: Serialize + DeserializeOwned + Clone> {
    pub(crate) key: S::Key,
    pub(crate) channel: Option<UseChannel<StorageChannelPayload<S>>>,
    pub(crate) data: T,
}

impl<S: StorageBacking, T: Display + Serialize + DeserializeOwned + Clone> Display
    for StorageEntry<S, T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

impl<S: StorageBacking, T: Debug + Serialize + DeserializeOwned + Clone> Debug
    for StorageEntry<S, T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

impl<S, T> StorageEntry<S, T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Clone + 'static,
    S::Key: Clone,
{
    pub fn new(key: S::Key, data: T, cx: Option<&ScopeState>) -> Self {
        let channel = {
            #[cfg(feature = "ssr")]
            {
                None
            }
            #[cfg(not(feature = "ssr"))]
            {
                cx.map_or_else(|| None, |cx| S::subscribe::<T>(cx, &key))
            }
        };
        Self { key, data, channel }
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

    pub fn update(&mut self) {
        self.data = S::get(&self.key).unwrap_or(self.data.clone());
    }
}

impl<S: StorageBacking, T: Serialize + DeserializeOwned + Clone> Deref for StorageEntry<S, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<S: StorageBacking, T: Serialize + DeserializeOwned + Clone> Drop for StorageEntry<S, T> {
    fn drop(&mut self) {}
}

pub struct StorageEntryMut<'a, S, T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Clone + 'static,
    S::Key: Clone,
{
    storage_entry: &'a mut StorageEntry<S, T>,
}

impl<'a, S, T> Deref for StorageEntryMut<'a, S, T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Clone,
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
    T: Serialize + DeserializeOwned + Clone,
    S::Key: Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.storage_entry.data
    }
}

impl<'a, S, T> Drop for StorageEntryMut<'a, S, T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Clone + 'static,
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

pub fn synced_storage_entry<S, T>(
    key: S::Key,
    init: impl FnOnce() -> T,
    cx: Option<&ScopeState>,
) -> StorageEntry<S, T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Clone + 'static,
    S::Key: Clone,
{
    let data = storage_entry::<S, T>(key.clone(), init);

    StorageEntry::new(key, data, cx)
}

pub fn use_synced_storage_entry<S, T>(
    cx: &ScopeState,
    key: S::Key,
    init: impl FnOnce() -> T,
) -> Signal<StorageEntry<S, T>>
where
    S: StorageBacking + 'static,
    T: Serialize + DeserializeOwned + Clone + 'static,
    S::Key: Clone,
{
    let state = use_signal(cx, || synced_storage_entry::<S, T>(key, init, Some(cx)));

    if let Some(channel) = &state.read().channel {
        use_listen_channel(cx, channel, move |_| async move { state.write().update() });
    }
    state
}
