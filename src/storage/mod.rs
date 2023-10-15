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

/// A storage hook that can be used to store data across application reloads.
/// 
/// It returns a Signal that can be used to read and modify the state.
/// The changes to the state will be persisted across reloads.
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

/// A storage hook that can be used to store data that will persist across application reloads and be synced across all app sessions for a given installation or browser.
/// 
/// This hook returns a Signal that can be used to read and modify the state.
/// The changes to the state will be persisted to storage and all other app sessions will be notified of the change to update their local state.
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
    let storage_entry = cx.use_hook(|| storage_entry_with_channel::<S, T>(key, init, cx));
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

/// Returns a StorageEntry with the latest value from storage or the init value if it doesn't exist.
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

/// Returns a synced StorageEntry with the latest value from storage or the init value if it doesn't exist.
/// 
/// This differs from `storage_entry` in that this one will return a channel to subscribe to updates to the underlying storage. 
pub fn storage_entry_with_channel<S, T>(
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
    StorageEntry::new_with_channel(key, data, cx)
}

/// Returns a value from storage or the init value if it doesn't exist.
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

/// A storage entry that can be used to store data across application reloads. It optionally provides a channel to subscribe to updates to the underlying storage.
#[derive(Clone, Default)]
pub struct StorageEntry<S: StorageBacking, T: Serialize + DeserializeOwned + Clone + 'static> {
    /// The key used to store the data in storage
    pub(crate) key: S::Key,
    /// A signal that can be used to read and modify the state
    pub(crate) data: Signal<T>,
    /// An optional channel to subscribe to updates to the underlying storage
    pub(crate) channel: Option<UseChannel<StorageChannelPayload<S>>>,
    /// A lock to prevent multiple saves from happening at the same time
    pub(crate) storage_save_lock: Arc<Mutex<()>>,
}

impl<S, T> StorageEntry<S, T>
where
    S: StorageBacking + StorageSubscriber<S>,
    T: Serialize + DeserializeOwned + Clone + 'static,
    S::Key: Clone,
{
    /// Creates a new StorageEntry with a channel to subscribe to updates to the underlying storage
    fn new_with_channel(key: S::Key, data: T, cx: &ScopeState) -> Self {
        let channel = S::subscribe::<T>(cx, &key);

        Self {
            key,
            data: Signal::new_in_scope(data, cx.scope_id()),
            channel,
            storage_save_lock: Arc::new(Mutex::new(())),
        }
    }
}

impl<S, T> StorageEntry<S, T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Clone + 'static,
    S::Key: Clone,
{
    /// Creates a new StorageEntry
    pub fn new(key: S::Key, data: T, cx: &ScopeState) -> Self {
        Self {
            key,
            data: Signal::new_in_scope(data, cx.scope_id()),
            channel: None,
            storage_save_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Saves the current state to storage. Only one save can happen at a time.
    pub(crate) fn save(&self) {
        let _ = self.storage_save_lock.try_lock().map(|_| {
            S::set(self.key.clone(), &self.data);
        });
    }

    /// Updates the state from storage
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

/// A trait for a storage backing
pub trait StorageBacking: Sized + Clone + 'static {
    /// The key type used to store data in storage
    type Key: Eq + PartialEq + Clone + Debug;
    /// Gets a value from storage for the given key
    fn get<T: DeserializeOwned>(key: &Self::Key) -> Option<T>;
    /// Sets a value in storage for the given key
    fn set<T: Serialize>(key: Self::Key, value: &T);
}

/// A trait for a subscriber to events from a storage backing
pub trait StorageSubscriber<S: StorageBacking> {
    /// Subscribes to events from a storage backing for the given key
    fn subscribe<T: DeserializeOwned + 'static>(
        cx: &ScopeState,
        key: &S::Key,
    ) -> Option<UseChannel<StorageChannelPayload<S>>>;
    /// Unsubscribes from events from a storage backing for the given key
    fn unsubscribe(key: &S::Key);
}
// End Storage Backing traits

// Start StorageChannelPayload

/// A payload for a storage channel that contains the key that was updated
#[derive(Clone)]
pub struct StorageChannelPayload<S: StorageBacking> {
    /// The key that was updated in storage
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
