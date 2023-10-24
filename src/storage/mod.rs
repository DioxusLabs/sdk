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
use dioxus_signals::{use_signal, Signal};
use postcard::to_allocvec;
use serde::{de::DeserializeOwned, Serialize};
use std::any::Any;
use std::fmt::{Debug, Display};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use tokio::sync::watch::{Receiver, Sender};

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

/// A storage hook that can be used to store data that will persist across application reloads.
///
/// This hook returns a Signal that can be used to read and modify the state.
pub fn use_storage<S, T>(cx: &ScopeState, key: S::Key, init: impl FnOnce() -> T) -> Signal<T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + PartialEq + 'static,
    S::Key: Clone,
{
    let mut init = Some(init);
    let signal = {
        if cfg!(feature = "ssr") {
            use_signal(cx, init.take().unwrap())
        } else if cfg!(feature = "hydrate") {
            let key_clone = key.clone();
            let storage_entry =
                cx.use_hook(|| storage_entry::<S, T>(key, init.take().unwrap(), cx));
            if cx.generation() == 0 {
                cx.needs_update();
            }
            if cx.generation() == 1 {
                storage_entry.set(get_from_storage::<S, T>(key_clone, init.take().unwrap()));
                storage_entry.use_save_to_storage_on_change(cx);
            }
            storage_entry.data
        } else {
            let storage_entry = use_storage_entry::<S, T>(cx, key, init.take().unwrap());
            storage_entry.use_save_to_storage_on_change(cx);
            storage_entry.data
        }
    };
    signal
}

/// A storage hook that can be used to store data that will persist across application reloads and be synced across all app sessions for a given installation or browser.
///
/// This hook returns a Signal that can be used to read and modify the state.
/// The changes to the state will be persisted to storage and all other app sessions will be notified of the change to update their local state.
pub fn use_synced_storage<S, T>(cx: &ScopeState, key: S::Key, init: impl FnOnce() -> T) -> Signal<T>
where
    S: StorageBacking + StorageSubscriber<S>,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + PartialEq + 'static,
    S::Key: Clone,
{
    let mut init = Some(init);
    let signal = {
        if cfg!(feature = "ssr") {
            use_signal(cx, init.take().unwrap())
        } else if cfg!(feature = "hydrate") {
            let key_clone = key.clone();
            let storage_entry =
                cx.use_hook(|| synced_storage_entry::<S, T>(key, init.take().unwrap(), cx));
            if cx.generation() == 0 {
                cx.needs_update();
            }
            if cx.generation() == 1 {
                storage_entry
                    .entry
                    .set(get_from_storage::<S, T>(key_clone, init.take().unwrap()));
                storage_entry.use_save_to_storage_on_change(cx);
                use_subscribe_to_storage(cx, storage_entry);
            }
            *storage_entry.data()
        } else {
            let storage_entry = use_synced_storage_entry::<S, T>(cx, key, init.take().unwrap());
            storage_entry.use_save_to_storage_on_change(cx);
            use_subscribe_to_storage(cx, storage_entry);
            *storage_entry.data()
        }
    };
    signal
}

/// A hook that creates a StorageEntry with the latest value from storage or the init value if it doesn't exist.
pub fn use_storage_entry<S, T>(
    cx: &ScopeState,
    key: S::Key,
    init: impl FnOnce() -> T,
) -> &mut StorageEntry<S, T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + PartialEq + 'static,
    S::Key: Clone,
{
    cx.use_hook(|| storage_entry::<S, T>(key, init, cx))
}

/// A hook that creates a StorageEntry with the latest value from storage or the init value if it doesn't exist, and provides a channel to subscribe to updates to the underlying storage.
pub fn use_synced_storage_entry<S, T>(
    cx: &ScopeState,
    key: S::Key,
    init: impl FnOnce() -> T,
) -> &mut SyncedStorageEntry<S, T>
where
    S: StorageBacking + StorageSubscriber<S>,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + PartialEq + 'static,
    S::Key: Clone,
{
    cx.use_hook(|| synced_storage_entry::<S, T>(key, init, cx))
}

/// A hook that will update the state from storage when the StorageEntry channel receives an update.
pub(crate) fn use_subscribe_to_storage<S, T>(
    cx: &ScopeState,
    storage_entry: &SyncedStorageEntry<S, T>,
) where
    S: StorageBacking + StorageSubscriber<S>,
    T: Serialize + DeserializeOwned + Clone + PartialEq + Send + Sync + 'static,
    S::Key: Clone,
{
    let storage_entry_signal = storage_entry.entry.data;
    let channel = storage_entry.channel.clone();
    use_effect(cx, (), move |_| async move {
        to_owned![channel];
        loop {
            if channel.changed().await.is_ok() {
                let payload = channel.borrow_and_update();
                *storage_entry_signal.write() = payload
                    .data
                    .downcast_ref::<T>()
                    .expect("Type mismatch with storage entry")
                    .clone();
            }
        }
    });
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
pub fn synced_storage_entry<S, T>(
    key: S::Key,
    init: impl FnOnce() -> T,
    cx: &ScopeState,
) -> SyncedStorageEntry<S, T>
where
    S: StorageBacking + StorageSubscriber<S>,
    T: Serialize + DeserializeOwned + Clone + PartialEq + Send + Sync + 'static,
    S::Key: Clone,
{
    let data = get_from_storage::<S, T>(key.clone(), init);
    SyncedStorageEntry::new(key, data, cx)
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

pub trait StorageEntryTrait<S: StorageBacking, T: PartialEq + Clone + 'static>:
    Clone + 'static
{
    /// Saves the current state to storage
    fn save(&self);

    /// Updates the state from storage
    fn update(&mut self);

    /// Gets the key used to store the data in storage
    fn key(&self) -> &S::Key;

    /// Gets the signal that can be used to read and modify the state
    fn data(&self) -> &Signal<T>;

    fn use_save_to_storage_on_change(&self, cx: &ScopeState)
    where
        S: StorageBacking,
        T: Serialize + DeserializeOwned + Clone + PartialEq + 'static,
    {
        let entry_clone = self.clone();
        use_effect(cx, (&self.data().value(),), move |_| async move {
            log::info!("state value changed, trying to save");
            entry_clone.save();
        });
    }
}

// Start SyncedStorageEntry

#[derive(Clone)]
pub struct SyncedStorageEntry<
    S: StorageBacking + StorageSubscriber<S>,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + PartialEq + 'static,
> {
    pub(crate) entry: StorageEntry<S, T>,
    pub(crate) channel: Receiver<StorageChannelPayload>,
}

impl<S, T> SyncedStorageEntry<S, T>
where
    S: StorageBacking + StorageSubscriber<S>,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + PartialEq + 'static,
{
    pub fn new(key: S::Key, data: T, cx: &ScopeState) -> Self {
        let channel = S::subscribe::<T>(cx, &key);
        Self {
            entry: StorageEntry::new(key, data, cx),
            channel,
        }
    }

    /// Gets the channel to subscribe to updates to the underlying storage
    pub fn channel(&self) -> Receiver<StorageChannelPayload> {
        self.channel.clone()
    }

    pub fn use_subscribe_to_storage(&self, cx: &ScopeState) {
        let storage_entry_signal = *self.data();
        let channel = self.channel.clone();
        use_effect(cx, (), move |_| async move {
            to_owned![channel, storage_entry_signal];
            loop {
                if channel.changed().await.is_ok() {
                    let payload = channel.borrow_and_update();
                    *storage_entry_signal.write() = payload
                        .data
                        .downcast_ref::<T>()
                        .expect("Type mismatch with storage entry")
                        .clone();
                }
            }
        });
    }
}

impl<S, T> StorageEntryTrait<S, T> for SyncedStorageEntry<S, T>
where
    S: StorageBacking + StorageSubscriber<S>,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + PartialEq + 'static,
{
    fn save(&self) {
        //  We want to save in the following conditions
        //      - The value from the channel is different from the current value
        //      - The value from the channel could not be determined, likely because it hasn't been set yet
        if let Some(payload) = self.channel.borrow().data.downcast_ref::<T>() {
            if *self.entry.data.read() == *payload {
                log::info!("value is the same, not saving");
                return
            }
        }
        log::info!("saving");
        self.entry.save();
    }

    fn update(&mut self) {
        self.entry.update();
    }

    fn key(&self) -> &S::Key {
        self.entry.key()
    }

    fn data(&self) -> &Signal<T> {
        &self.entry.data
    }
}

// Start StorageEntry

/// A storage entry that can be used to store data across application reloads. It optionally provides a channel to subscribe to updates to the underlying storage.
#[derive(Clone)]
pub struct StorageEntry<S: StorageBacking, T: Serialize + DeserializeOwned + Clone + 'static> {
    /// The key used to store the data in storage
    pub(crate) key: S::Key,
    /// A signal that can be used to read and modify the state
    pub(crate) data: Signal<T>,
    /// An optional channel to subscribe to updates to the underlying storage
    /// A lock to prevent multiple saves from happening at the same time
    storage_save_lock: Arc<Mutex<()>>, // TODO: probably unnecessary
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
            storage_save_lock: Arc::new(Mutex::new(())),
        }
    }
}

impl<S, T> StorageEntryTrait<S, T> for StorageEntry<S, T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Clone + PartialEq + 'static,
{
    fn save(&self) {
        let _ = self.storage_save_lock.try_lock().map(|_| {
            S::set(self.key.clone(), &self.data);
        });
    }

    fn update(&mut self) {
        self.data = S::get(&self.key).unwrap_or(self.data);
    }

    fn key(&self) -> &S::Key {
        &self.key
    }

    fn data(&self) -> &Signal<T> {
        &self.data
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
pub trait StorageBacking: Clone + 'static {
    /// The key type used to store data in storage
    type Key: PartialEq + Clone + Debug + Send + Sync + 'static;
    /// Gets a value from storage for the given key
    fn get<T: DeserializeOwned>(key: &Self::Key) -> Option<T>;
    /// Sets a value in storage for the given key
    fn set<T: Serialize>(key: Self::Key, value: &T);
}

/// A trait for a subscriber to events from a storage backing
pub trait StorageSubscriber<S: StorageBacking> {
    /// Subscribes to events from a storage backing for the given key
    fn subscribe<T: DeserializeOwned + Send + Sync + 'static>(
        cx: &ScopeState,
        key: &S::Key,
    ) -> Receiver<StorageChannelPayload>;
    /// Unsubscribes from events from a storage backing for the given key
    fn unsubscribe(key: &S::Key);
}
// End Storage Backing traits

// Start StorageChannelPayload

/// A payload for a storage channel that contains the key that was updated
#[derive(Clone, Debug)]
pub struct StorageChannelPayload {
    data: Arc<dyn Any + Send + Sync>,
}

impl StorageChannelPayload {
    /// Creates a new StorageChannelPayload
    pub fn new<T: Send + Sync + 'static>(data: T) -> Self {
        Self {
            data: Arc::new(data),
        }
    }

    /// Gets the data from the payload
    pub fn data<T: 'static>(&self) -> Option<&T> {
        self.data.downcast_ref::<T>()
    }
}

impl Default for StorageChannelPayload {
    fn default() -> Self {
        Self { data: Arc::new(()) }
    }
}
// End StorageChannelPayload

pub struct StorageSenderEntry {
    pub(crate) getter: Box<dyn Fn() -> StorageChannelPayload + 'static + Send + Sync>,
    pub(crate) tx: Sender<StorageChannelPayload>,
}

impl StorageSenderEntry {
    pub fn new<
        S: StorageBacking + StorageSubscriber<S>,
        T: DeserializeOwned + Send + Sync + 'static,
    >(
        tx: Sender<StorageChannelPayload>,
        key: S::Key,
    ) -> Self {
        let getter = move || {
            let data = S::get::<T>(&key).unwrap();
            StorageChannelPayload::new(data)
        };
        Self {
            getter: Box::new(getter),
            tx,
        }
    }
}

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
