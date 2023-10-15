//! # dioxus-storage
//! A library for handling local storage ergonomically in Dioxus
//! ## Usage
//! ```rust
//! use dioxus_storage::use_storage;
//! use dioxus::prelude::*;
//! fn main() {
//!     dioxus_web::launch(app)
//! }
//!
//! fn app(cx: Scope) -> Element {
//!     let num = use_persistent(cx, "count", || 0);
//!     cx.render(rsx! {
//!         div {
//!             button {
//!                 onclick: move |_| {
//!                     num.modify(|num| *num += 1);
//!                 },
//!                 "Increment"
//!             }
//!             div {
//!                 "{*num.read()}"
//!             }
//!         }
//!     })
//! }
//! ```

mod client_storage;
mod persistence;

pub use client_storage::{LocalStorage, SessionStorage};
pub use persistence::{use_persistent, use_singleton_persistent};

use dioxus::prelude::{to_owned, use_effect, ScopeState};
use dioxus_signals::Signal;
use postcard::to_allocvec;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::{Debug, Display};
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use crate::utils::channel::{use_listen_channel, UseChannel};

#[cfg(not(target_family = "wasm"))]
pub use client_storage::set_dir;

// // Start use_storage hooks
// pub fn use_synced_storage<T>(
//     cx: &ScopeState,
//     key: impl ToString,
//     init: impl FnOnce() -> T,
// ) -> &mut Signal<T>
// where
//     T: Serialize + DeserializeOwned + Clone + PartialEq + 'static,
// {
//     cfg_if::cfg_if! {

//     }
// }

pub fn use_storage_entry<S, T>(
    cx: &ScopeState,
    key: S::Key,
    init: impl FnOnce() -> T,
) -> &mut StorageEntry<S,T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Clone + PartialEq + 'static,
    S::Key: Clone,
{
    let storage_entry = cx.use_hook(|| {
        storage_entry::<S,T>(key, init, cx)
    });

    let storage_entry_clone: StorageEntry<S, T> = storage_entry.clone();
    use_effect(cx, (&storage_entry_clone.data.value(),), move |_| async move {
        log::info!("state value changed, trying to save");
        storage_entry_clone.save();
    });

    storage_entry
}

#[allow(unused)]
pub fn use_storage_entry_with_subscription<S, T>(
    cx: &ScopeState,
    key: S::Key,
    init: impl FnOnce() -> T,
) -> &mut StorageEntry<S,T>
where
    S: StorageBacking + StorageSubscriber<S>,
    T: Serialize + DeserializeOwned + Clone + PartialEq + 'static,
    S::Key: Clone,
{
    let key_clone = key.clone();
    let storage_entry = cx.use_hook(|| synced_storage_entry::<S, T>(key, init, cx));
    let storage_entry_signal = storage_entry.data;
    if let Some(channel) = storage_entry.channel.clone() {
        use_listen_channel(cx, &channel, move |message| {
            to_owned![key_clone];
            async move {
                if let Ok(payload) = message {
                    if payload.key == key_clone {
                        *storage_entry_signal.write() =
                            get_from_storage::<S, T>(key_clone, || storage_entry_signal.value())
                    }
                }
            }
        });
    }

    let storage_entry_clone: StorageEntry<S, T> = storage_entry.clone();
    use_effect(cx, (&storage_entry_clone.data.value(),), move |_| async move {
        log::info!("state value changed, trying to save");
        storage_entry_clone.save();
    });
    storage_entry
}

pub fn storage_entry<S, T>(
    key: S::Key,
    init: impl FnOnce() -> T,
    cx: &ScopeState,
) -> StorageEntry<S, T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Clone + 'static,
    S::Key: Clone,
{
    let data = get_from_storage::<S, T>(key.clone(), init);
    StorageEntry::new(key, data, cx)
}

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
    let data = get_from_storage::<S, T>(key.clone(), init);
    StorageEntry::new_synced(key, data, cx)
}

pub fn get_from_storage<S: StorageBacking, T: Serialize + DeserializeOwned>(
    key: S::Key,
    init: impl FnOnce() -> T,
) -> T {
    S::get(&key).unwrap_or_else(|| {
        let data = init();
        S::set(key, &data);
        data
    })
}
// End use_storage hooks

// Start StorageEntry
#[derive(Clone, Default)]
pub struct StorageEntry<S: StorageBacking, T: Serialize + DeserializeOwned + Clone + 'static> {
    pub(crate) key: S::Key,
    pub(crate) data: Signal<T>,
    pub(crate) channel: Option<UseChannel<StorageChannelPayload<S>>>,
    pub(crate) lock: Arc<Mutex<()>>,
}

impl<S, T> StorageEntry<S, T>
where
    S: StorageBacking + StorageSubscriber<S>,
    T: Serialize + DeserializeOwned + Clone + 'static,
    S::Key: Clone,
{
    fn new_synced(key: S::Key, data: T, cx: &ScopeState) -> Self {
        let channel = S::subscribe::<T>(cx, &key);

        Self {
            key,
            data: Signal::new_in_scope(data, cx.scope_id()),
            channel,
            lock: Arc::new(Mutex::new(())),
        }
    }
}

impl<S, T> StorageEntry<S, T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Clone + 'static,
    S::Key: Clone,
{
    pub fn new(key: S::Key, data: T, cx: &ScopeState) -> Self {
        Self {
            key,
            data: Signal::new_in_scope(data, cx.scope_id()),
            channel: None,
            lock: Arc::new(Mutex::new(())),
        }
    }

    pub(crate) fn save(&self) {
        let _ = self.lock.try_lock().map(|_| {
            S::set(self.key.clone(), &self.data);
        });
    }

    pub fn update(&mut self) {
        self.data = S::get(&self.key).unwrap_or(self.data);
    }
}

impl<S: StorageBacking, T: Serialize + DeserializeOwned + Clone> Deref for StorageEntry<S, T> {
    type Target = Signal<T>;

    fn deref(&self) -> &Signal<T> {
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
// End StorageEntry

// Start Storage Backing traits
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
// End Storage Backing traits

// Start StorageChannelPayload
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
// End StorageChannelPayload

// Start helper functions
pub(crate) fn serde_to_string<T: Serialize>(value: &T) -> String {
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
pub(crate) fn serde_from_string<T: DeserializeOwned>(value: &str) -> T {
    try_serde_from_string(value).unwrap()
}

pub(crate) fn try_serde_from_string<T: DeserializeOwned>(value: &str) -> Option<T> {
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
// End helper functions
