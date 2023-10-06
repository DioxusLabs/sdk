use async_broadcast::{broadcast, Receiver, Sender, InactiveReceiver};
use dioxus::prelude::{ScopeState, use_future, use_context};
use dioxus_signals::{Signal, use_signal};
use postcard::to_allocvec;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, RwLock, Arc};
use std::hash::Hash;
use uuid::Uuid;

use crate::utils::channel::{UseChannel, use_channel, use_listen_channel};

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
    fn subscribe<T: DeserializeOwned + 'static>(cx: &ScopeState, key: &Self::Key) -> Option<UseChannel<StorageChannelPayload>>;
    fn get<T: DeserializeOwned>(key: &Self::Key) -> Option<T>;
    fn set<T: Serialize>(key: Self::Key, value: &T);
    fn get_subscriptions(cx: &ScopeState) -> StorageBackingSubscriptions<Self> {
        cx.consume_context::<StorageBackingSubscriptions<Self>>().unwrap_or_else(|| cx.provide_root_context(StorageBackingSubscriptions::<Self>::new()))
    }
}

pub(crate) fn do_storage_backing_subscribe<S: StorageBacking + Sized + 'static, T: 'static>(cx: &ScopeState, key: &S::Key) -> Option<UseChannel<StorageChannelPayload>> {
    log::info!("Subscribing to storage entry: {:?}", key);
    #[cfg(not(feature = "ssr"))]
    {
        let subscriptions = S::get_subscriptions(cx);
        if let Some(storage_sender) = subscriptions.get(key) {
            if storage_sender.data_type_id == TypeId::of::<T>() {
                log::info!("Already subscribed to storage entry: {:?}", key);
                Some(storage_sender.channel.clone())
            } else {
                log::info!("Already subscribed to storage entry: {:?} but with a different type", key);
                None
            }
        } else {
            let (tx, rx) = broadcast::<StorageChannelPayload>(5);
            subscriptions.insert(
                key.clone(),
                StorageSender {
                    channel: UseChannel::new(Uuid::new_v4(), tx, rx.deactivate()),
                    data_type_id: TypeId::of::<T>(),
                },
            );
            log::info!("Subscribed to storage entry: {:?}", key);
            subscriptions.get(key).map(|storage_sender| storage_sender.channel.clone())
        }
    }
    #[cfg(feature = "ssr")]
    {
        log::info!("Subscription not supported for SSR");
        None
    }
}

#[derive(Clone)]
pub(crate) struct StorageBackingSubscriptions<S: StorageBacking + 'static> {
    pub(crate) subscriptions: Arc<RwLock<HashMap<S::Key, StorageSender>>>,
}

impl<S: StorageBacking + 'static> StorageBackingSubscriptions<S> {
    pub(crate) fn new() -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub(crate) fn get(&self, key: &S::Key) -> Option<StorageSender> {
        self.subscriptions.read().unwrap().get(key).map_or( None, |storage_sender| Some((*storage_sender).clone()))
    }
    pub(crate) fn insert(&self, key: S::Key, storage_sender: StorageSender) {
        if let Some(existing_sender) = self.get(&key) {
            if existing_sender.data_type_id != storage_sender.data_type_id {
                panic!("Storage sender type mismatch");
            }
        } else {
            self.subscriptions.write().unwrap().insert(key, storage_sender);
        }
    }
}

#[derive(Clone)]
pub struct StorageSender {
    pub(crate) channel: UseChannel<StorageChannelPayload>,
    pub(crate) data_type_id: TypeId,
}

#[derive(Clone)]
pub enum StorageChannelPayload {
    Updated,
}

#[derive(Clone, Default)]
pub struct StorageEntry<S: StorageBacking, T: Serialize + DeserializeOwned + Clone> {
    key: S::Key,
    pub(crate) channel: StorageEntryChannel,
    pub(crate) data: T,
}

impl<S: StorageBacking, T: Display + Serialize + DeserializeOwned + Clone> Display for StorageEntry<S, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

impl<S: StorageBacking, T: Debug + Serialize + DeserializeOwned + Clone> Debug for StorageEntry<S, T> {
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
                StorageEntryChannel::default()
            }
            #[cfg(not(feature = "ssr"))]
            {
                match cx {
                    Some(cx) => StorageEntryChannel::new(S::subscribe::<T>(cx, &key)),
                    None => StorageEntryChannel::default(),
                }
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
    fn drop(&mut self) {
        
    }
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

#[derive(Clone, Default)]
pub(crate) enum StorageEntryChannel {
    #[default]
    None,
    Active(UseChannel<StorageChannelPayload>),
}

impl StorageEntryChannel {
    fn new(receiver: Option<UseChannel<StorageChannelPayload>>) -> Self {
        match receiver {
            Some(receiver) => Self::Active(receiver),
            None => Self::None,
        }
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

pub fn synced_storage_entry<S, T>(key: S::Key, init: impl FnOnce() -> T, cx: Option<&ScopeState>) -> StorageEntry<S, T>
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
    let state = use_signal(cx, || {
        synced_storage_entry::<S, T>(key, init, Some(cx))
    });

    if let StorageEntryChannel::Active(channel) = &state.read().channel {
        use_listen_channel(cx, channel, move |_| async move { state.write().update() });
    }
    state
}
