use dioxus::prelude::ScopeState;
use dioxus_signals::{use_signal, Signal};
use postcard::to_allocvec;
use serde::{de::DeserializeOwned, Serialize};
use std::cell::Ref;
use std::fmt::{Debug, Display};
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

#[allow(unused)]
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

pub trait StorageBacking: Sized + Clone + 'static {
    type Key: Eq + PartialEq + Clone + Debug;
    fn get<T: DeserializeOwned>(key: &Self::Key) -> Option<T>;
    fn set<T: Serialize>(key: Self::Key, value: &T);
}

pub trait StorageSubscriber<S: StorageBacking> {
    fn subscribe<T: DeserializeOwned + 'static>(
        cx: &ScopeState,
        key: &S::Key,
    ) -> Option<UseChannel<StorageChannelPayload<S>>>;
    fn unsubscribe(key: &S::Key);
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
    pub(crate) data: T,
    pub(crate) channel: Option<UseChannel<StorageChannelPayload<S>>>,
}

impl<S, T> StorageEntry<S, T>
where
    S: StorageBacking + StorageSubscriber<S>,
    T: Serialize + DeserializeOwned + Clone + 'static,
    S::Key: Clone,
{
    fn new_synced(key: S::Key, data: T, cx: &ScopeState) -> Self {
        let channel = S::subscribe::<T>(cx, &key);
        Self { key, data, channel }
    }
}

impl<S, T> StorageEntry<S, T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Clone + 'static,
    S::Key: Clone,
{
    pub fn new(key: S::Key, data: T) -> Self {
        Self {
            key,
            data,
            channel: None,
        }
    }

    pub(crate) fn save(&self) {
        S::set(self.key.clone(), &self.data);
    }

    pub fn with_mut(&mut self, f: impl FnOnce(&mut T)) {
        f(&mut self.data);
        self.save()
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

#[allow(unused)]
pub fn synced_storage_entry<S, T>(
    key: S::Key,
    init: impl FnOnce() -> T,
    cx: &ScopeState,
) -> StorageEntry<S, T>
where
    S: StorageBacking + StorageSubscriber<S>,
    T: Serialize + DeserializeOwned + Clone + 'static,
    S::Key: Clone,
{
    let data = storage_entry::<S, T>(key.clone(), init);

    StorageEntry::new_synced(key, data, cx)
}

#[allow(unused)]
pub fn use_synced_storage_entry<S, T>(
    cx: &ScopeState,
    key: S::Key,
    init: impl FnOnce() -> T,
) -> UseStorageEntry<S, T>
where
    S: StorageBacking + StorageSubscriber<S>,
    T: Serialize + DeserializeOwned + Clone + 'static,
    S::Key: Clone,
{
    let state = use_signal(cx, || synced_storage_entry::<S, T>(key, init, cx));

    if let Some(channel) = &state.read().channel {
        use_listen_channel(cx, channel, move |message| async move {
            if let Ok(payload) = message {
                if payload.key == state.read().key {
                    state.write().update()
                }
            }
        });
    }
    UseStorageEntry::new(state)
}

//  Start UseStorageEntry
/// Storage that persists across application reloads
pub struct UseStorageEntry<S: StorageBacking, T: Serialize + DeserializeOwned + Clone + 'static> {
    inner: Signal<StorageEntry<S, T>>,
}

#[allow(unused)]
impl<S: StorageBacking, T: Serialize + DeserializeOwned + Clone + 'static> UseStorageEntry<S, T> {
    pub fn new(signal: Signal<StorageEntry<S, T>>) -> Self {
        Self { inner: signal }
    }

    /// Returns a reference to the value
    pub fn read(&self) -> Ref<T> {
        Ref::map(self.inner.read(), |entry| &entry.data)
    }

    /// Sets the value
    pub fn set(&self, value: T) {
        self.inner.write().with_mut(|data| *data = value)
    }

    /// Modifies the value
    pub fn modify<F: FnOnce(&mut T)>(&self, f: F) {
        let writer = &mut *self.inner.write();
        writer.with_mut(f);
    }
}

#[allow(unused)]
impl<S: StorageBacking, T: Serialize + DeserializeOwned + Default + Clone + 'static>
    UseStorageEntry<S, T>
{
    /// Returns a clone of the value
    pub fn get(&self) -> T {
        self.read().clone()
    }
}

impl<S: StorageBacking, T: Serialize + DeserializeOwned + Default + Display + Clone + 'static>
    Display for UseStorageEntry<S, T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (*self.read()).fmt(f)
    }
}

impl<S: StorageBacking, T: Serialize + DeserializeOwned + Default + Clone + 'static> Deref
    for UseStorageEntry<S, T>
{
    type Target = Signal<StorageEntry<S, T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S: StorageBacking, T: Serialize + DeserializeOwned + Default + Clone + 'static> DerefMut
    for UseStorageEntry<S, T>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// End UseStorageEntry

// state.with(move |state| {
//     if let Some(channel) = &state.channel {

//         use_listen_channel(cx, channel, move |message: Result<StorageChannelPayload<S>, async_broadcast::RecvError>| async move {
//             if let Ok(payload) = message {
//                 if state.key == payload.key {
//                     state.update();
//                 }
//             }
//         });
//     }
// });
// state
