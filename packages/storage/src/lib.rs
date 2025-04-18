//! Local and persistent storage.
//!
//! Handle local storage ergonomically.
//!
//! ## Usage
//! ```rust
//! use dioxus_storage::use_persistent;
//! use dioxus::prelude::*;
//!
//! #[component]
//! fn App() -> Element {
//!     let mut num = use_persistent("count", || 0);
//!     rsx! {
//!         div {
//!             button {
//!                 onclick: move |_| {
//!                     *num.write() += 1;
//!                 },
//!                 "Increment"
//!             }
//!             div {
//!                 "{*num.read()}"
//!             }
//!         }
//!     }
//! }
//! ```

mod client_storage;
mod persistence;

pub use client_storage::{LocalStorage, SessionStorage};
use dioxus::logger::tracing::trace;
use futures_util::stream::StreamExt;
pub use persistence::{
    new_persistent, new_singleton_persistent, use_persistent, use_singleton_persistent,
};
use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;
use serde::{Serialize, de::DeserializeOwned};
use std::any::Any;
use std::fmt::{Debug, Display};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::watch::error::SendError;
use tokio::sync::watch::{Receiver, Sender};

#[cfg(not(target_family = "wasm"))]
pub use client_storage::{set_dir_name, set_directory};

/// A storage hook that can be used to store data that will persist across application reloads. This hook is generic over the storage location which can be useful for other hooks.
///
/// This hook returns a Signal that can be used to read and modify the state.
///
/// ## Usage
///
/// ```rust
/// use dioxus_storage::{use_storage, StorageBacking};
/// use dioxus::prelude::*;
/// use dioxus_signals::Signal;
///
/// // This hook can be used with any storage backing without multiple versions of the hook
/// fn use_user_id<S>() -> Signal<usize> where S: StorageBacking<Key=&'static str> {
///     use_storage::<S, _>("user-id", || 123)
/// }
/// ```
pub fn use_storage<S, T>(key: S::Key, init: impl FnOnce() -> T) -> Signal<T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + PartialEq + 'static,
    S::Key: Clone,
{
    let mut init = Some(init);
    let storage = use_hook(|| new_storage::<S, T>(key, || init.take().unwrap()()));
    use_hydrate_storage::<S, T>(storage, init);
    storage
}

#[allow(unused)]
enum StorageMode {
    Client,
    HydrateClient,
    Server,
}

impl StorageMode {
    // Get the active mode
    #[allow(unreachable_code)]
    const fn current() -> Self {
        server_only! {
            return StorageMode::Server;
        }

        fullstack! {
            return StorageMode::HydrateClient;
        }

        StorageMode::Client
    }
}

/// Creates a Signal that can be used to store data that will persist across application reloads.
///
/// This hook returns a Signal that can be used to read and modify the state.
///
/// ## Usage
///
/// ```rust
/// use dioxus_storage::{new_storage, StorageBacking};
/// use dioxus::prelude::*;
/// use dioxus_signals::Signal;
///
/// // This hook can be used with any storage backing without multiple versions of the hook
/// fn user_id<S>() -> Signal<usize> where S: StorageBacking<Key=&'static str> {
///     new_storage::<S, _>("user-id", || 123)
/// }
/// ```
pub fn new_storage<S, T>(key: S::Key, init: impl FnOnce() -> T) -> Signal<T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + PartialEq + 'static,
    S::Key: Clone,
{
    let mode = StorageMode::current();

    match mode {
        // SSR does not support storage on the backend. We will just use a normal Signal to represent the initial state.
        // The client will hydrate this with a correct StorageEntry and maintain state.
        StorageMode::Server => Signal::new(init()),
        _ => {
            // Otherwise the client is rendered normally, so we can just use the storage entry.
            let storage_entry = new_storage_entry::<S, T>(key, init);
            storage_entry.save_to_storage_on_change();
            storage_entry.data
        }
    }
}

/// A storage hook that can be used to store data that will persist across application reloads and be synced across all app sessions for a given installation or browser.
///
/// This hook returns a Signal that can be used to read and modify the state.
/// The changes to the state will be persisted to storage and all other app sessions will be notified of the change to update their local state.
pub fn use_synced_storage<S, T>(key: S::Key, init: impl FnOnce() -> T) -> Signal<T>
where
    S: StorageBacking + StorageSubscriber<S>,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + PartialEq + 'static,
    S::Key: Clone,
{
    let mut init = Some(init);
    let storage = use_hook(|| new_synced_storage::<S, T>(key, || init.take().unwrap()()));
    use_hydrate_storage::<S, T>(storage, init);
    storage
}

/// Create a signal that can be used to store data that will persist across application reloads and be synced across all app sessions for a given installation or browser.
///
/// This hook returns a Signal that can be used to read and modify the state.
/// The changes to the state will be persisted to storage and all other app sessions will be notified of the change to update their local state.
pub fn new_synced_storage<S, T>(key: S::Key, init: impl FnOnce() -> T) -> Signal<T>
where
    S: StorageBacking + StorageSubscriber<S>,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + PartialEq + 'static,
    S::Key: Clone,
{
    let signal = {
        let mode = StorageMode::current();

        match mode {
            // SSR does not support synced storage on the backend. We will just use a normal Signal to represent the initial state.
            // The client will hydrate this with a correct SyncedStorageEntry and maintain state.
            StorageMode::Server => Signal::new(init()),
            _ => {
                // The client is rendered normally, so we can just use the synced storage entry.
                let storage_entry = new_synced_storage_entry::<S, T>(key, init);
                storage_entry.save_to_storage_on_change();
                storage_entry.subscribe_to_storage();
                *storage_entry.data()
            }
        }
    };
    signal
}

/// A hook that creates a StorageEntry with the latest value from storage or the init value if it doesn't exist.
pub fn use_storage_entry<S, T>(key: S::Key, init: impl FnOnce() -> T) -> StorageEntry<S, T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + PartialEq + 'static,
    S::Key: Clone,
{
    let mut init = Some(init);
    let signal = use_hook(|| new_storage_entry::<S, T>(key, || init.take().unwrap()()));
    use_hydrate_storage::<S, T>(*signal.data(), init);
    signal
}

/// A hook that creates a StorageEntry with the latest value from storage or the init value if it doesn't exist, and provides a channel to subscribe to updates to the underlying storage.
pub fn use_synced_storage_entry<S, T>(
    key: S::Key,
    init: impl FnOnce() -> T,
) -> SyncedStorageEntry<S, T>
where
    S: StorageBacking + StorageSubscriber<S>,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + PartialEq + 'static,
    S::Key: Clone,
{
    let mut init = Some(init);
    let signal = use_hook(|| new_synced_storage_entry::<S, T>(key, || init.take().unwrap()()));
    use_hydrate_storage::<S, T>(*signal.data(), init);
    signal
}

/// Returns a StorageEntry with the latest value from storage or the init value if it doesn't exist.
pub fn new_storage_entry<S, T>(key: S::Key, init: impl FnOnce() -> T) -> StorageEntry<S, T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    S::Key: Clone,
{
    let data = get_from_storage::<S, T>(key.clone(), init);
    StorageEntry::new(key, data)
}

/// Returns a synced StorageEntry with the latest value from storage or the init value if it doesn't exist.
///
/// This differs from `storage_entry` in that this one will return a channel to subscribe to updates to the underlying storage.
pub fn new_synced_storage_entry<S, T>(
    key: S::Key,
    init: impl FnOnce() -> T,
) -> SyncedStorageEntry<S, T>
where
    S: StorageBacking + StorageSubscriber<S>,
    T: Serialize + DeserializeOwned + Clone + PartialEq + Send + Sync + 'static,
    S::Key: Clone,
{
    let data = get_from_storage::<S, T>(key.clone(), init);
    SyncedStorageEntry::new(key, data)
}

/// Returns a value from storage or the init value if it doesn't exist.
pub fn get_from_storage<
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Send + Sync + Clone + 'static,
>(
    key: S::Key,
    init: impl FnOnce() -> T,
) -> T {
    S::get(&key).unwrap_or_else(|| {
        let data = init();
        S::set(key, &data);
        data
    })
}

/// A trait for common functionality between StorageEntry and SyncedStorageEntry
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

    /// Creates a hook that will save the state to storage when the state changes
    fn save_to_storage_on_change(&self)
    where
        S: StorageBacking,
        T: Serialize + DeserializeOwned + Clone + PartialEq + 'static,
    {
        let entry_clone = self.clone();
        let old = RefCell::new(None);
        let data = *self.data();
        spawn(async move {
            loop {
                let (rc, mut reactive_context) = ReactiveContext::new();
                rc.run_in(|| {
                    if old.borrow().as_ref() != Some(&*data.read()) {
                        trace!("Saving to storage");
                        entry_clone.save();
                        old.replace(Some(data()));
                    }
                });
                if reactive_context.next().await.is_none() {
                    break;
                }
            }
        });
    }
}

/// A wrapper around StorageEntry that provides a channel to subscribe to updates to the underlying storage.
#[derive(Clone)]
pub struct SyncedStorageEntry<
    S: StorageBacking + StorageSubscriber<S>,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + PartialEq + 'static,
> {
    /// The underlying StorageEntry that is used to store the data and track the state
    pub(crate) entry: StorageEntry<S, T>,
    /// The channel to subscribe to updates to the underlying storage
    pub(crate) channel: Receiver<StorageChannelPayload>,
}

impl<S, T> SyncedStorageEntry<S, T>
where
    S: StorageBacking + StorageSubscriber<S>,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + PartialEq + 'static,
{
    pub fn new(key: S::Key, data: T) -> Self {
        let channel = S::subscribe::<T>(&key);
        Self {
            entry: StorageEntry::new(key, data),
            channel,
        }
    }

    /// Gets the channel to subscribe to updates to the underlying storage
    pub fn channel(&self) -> &Receiver<StorageChannelPayload> {
        &self.channel
    }

    /// Creates a hook that will update the state when the underlying storage changes
    pub fn subscribe_to_storage(&self) {
        let storage_entry_signal = *self.data();
        let channel = self.channel.clone();
        spawn(async move {
            to_owned![channel, storage_entry_signal];
            loop {
                // Wait for an update to the channel
                if channel.changed().await.is_ok() {
                    // Retrieve the latest value from the channel, mark it as read, and update the state
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
                return;
            }
        }
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

/// A storage entry that can be used to store data across application reloads. It optionally provides a channel to subscribe to updates to the underlying storage.
#[derive(Clone)]
pub struct StorageEntry<
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
> {
    /// The key used to store the data in storage
    pub(crate) key: S::Key,
    /// A signal that can be used to read and modify the state
    pub(crate) data: Signal<T>,
}

impl<S, T> StorageEntry<S, T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    S::Key: Clone,
{
    /// Creates a new StorageEntry
    pub fn new(key: S::Key, data: T) -> Self {
        Self {
            key,
            data: Signal::new_in_scope(
                data,
                current_scope_id().expect("must be called from inside of the dioxus context"),
            ),
        }
    }
}

impl<S, T> StorageEntryTrait<S, T> for StorageEntry<S, T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Clone + PartialEq + Send + Sync + 'static,
{
    fn save(&self) {
        S::set(self.key.clone(), &*self.data.read());
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

impl<S: StorageBacking, T: Serialize + DeserializeOwned + Clone + Send + Sync> Deref
    for StorageEntry<S, T>
{
    type Target = Signal<T>;

    fn deref(&self) -> &Signal<T> {
        &self.data
    }
}

impl<S: StorageBacking, T: Serialize + DeserializeOwned + Clone + Send + Sync> DerefMut
    for StorageEntry<S, T>
{
    fn deref_mut(&mut self) -> &mut Signal<T> {
        &mut self.data
    }
}

impl<S: StorageBacking, T: Display + Serialize + DeserializeOwned + Clone + Send + Sync> Display
    for StorageEntry<S, T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

impl<S: StorageBacking, T: Debug + Serialize + DeserializeOwned + Clone + Send + Sync> Debug
    for StorageEntry<S, T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

/// A trait for a storage backing
pub trait StorageBacking: Clone + 'static {
    /// The key type used to store data in storage
    type Key: PartialEq + Clone + Debug + Send + Sync + 'static;
    /// Gets a value from storage for the given key
    fn get<T: DeserializeOwned + Clone + 'static>(key: &Self::Key) -> Option<T>;
    /// Sets a value in storage for the given key
    fn set<T: Serialize + Send + Sync + Clone + 'static>(key: Self::Key, value: &T);
}

/// A trait for a subscriber to events from a storage backing
pub trait StorageSubscriber<S: StorageBacking> {
    /// Subscribes to events from a storage backing for the given key
    fn subscribe<T: DeserializeOwned + Send + Sync + Clone + 'static>(
        key: &S::Key,
    ) -> Receiver<StorageChannelPayload>;
    /// Unsubscribes from events from a storage backing for the given key
    fn unsubscribe(key: &S::Key);
}

/// A struct to hold information about processing a storage event.
pub struct StorageSubscription {
    /// A getter function that will get the data from storage and return it as a StorageChannelPayload.
    pub(crate) getter: Box<dyn Fn() -> StorageChannelPayload + 'static + Send + Sync>,

    /// The channel to send the data to.
    pub(crate) tx: Arc<Sender<StorageChannelPayload>>,
}

impl StorageSubscription {
    pub fn new<
        S: StorageBacking + StorageSubscriber<S>,
        T: DeserializeOwned + Send + Sync + Clone + 'static,
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
            tx: Arc::new(tx),
        }
    }

    /// Gets the latest data from storage and sends it to the channel.
    pub fn get_and_send(&self) -> Result<(), SendError<StorageChannelPayload>> {
        let payload = (self.getter)();
        self.tx.send(payload)
    }
}

/// A payload for a storage channel that contains the latest value from storage.
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

// Helper functions

/// Serializes a value to a string and compresses it.
pub(crate) fn serde_to_string<T: Serialize>(value: &T) -> String {
    let mut serialized = Vec::new();
    ciborium::into_writer(value, &mut serialized).unwrap();
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
/// Deserializes a value from a string and unwraps errors.
pub(crate) fn serde_from_string<T: DeserializeOwned>(value: &str) -> T {
    try_serde_from_string(value).unwrap()
}

/// Deserializes and decompresses a value from a string and returns None if there is an error.
pub(crate) fn try_serde_from_string<T: DeserializeOwned>(value: &str) -> Option<T> {
    let mut bytes: Vec<u8> = Vec::new();
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        let n1 = c.to_digit(16)?;
        let c2 = chars.next()?;
        let n2 = c2.to_digit(16)?;
        bytes.push((n1 * 16 + n2) as u8);
    }

    match yazi::decompress(&bytes, yazi::Format::Zlib) {
        Ok((decompressed, _)) => ciborium::from_reader(std::io::Cursor::new(decompressed)).ok(),
        Err(_) => None,
    }
}

// Take a signal and a storage key and hydrate the value if we are hydrating the client.
pub(crate) fn use_hydrate_storage<S, T>(
    mut signal: Signal<T>,
    init: Option<impl FnOnce() -> T>,
) -> Signal<T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + PartialEq + 'static,
    S::Key: Clone,
{
    let mode = StorageMode::current();
    // We read the value from storage and store it here if we are hydrating the client.
    let original_storage_value: Rc<RefCell<Option<T>>> = use_hook(|| Rc::new(RefCell::new(None)));

    // If we are not hydrating the client
    if let StorageMode::HydrateClient = mode {
        if generation() == 0 {
            // We always use the default value for the first render.
            if let Some(default_value) = init {
                // Read the value from storage before we reset it for hydration
                original_storage_value
                    .borrow_mut()
                    .replace(signal.peek().clone());
                signal.set(default_value());
            }
            // And we trigger a new render for after hydration
            needs_update();
        }
        if generation() == 1 {
            // After we hydrate, set the original value from storage
            if let Some(original_storage_value) = original_storage_value.borrow_mut().take() {
                signal.set(original_storage_value);
            }
        }
    }
    signal
}
