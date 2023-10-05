use async_broadcast::{broadcast, Receiver, Sender, InactiveReceiver};
use dioxus::prelude::{ScopeState, use_future};
use dioxus_signals::{Signal, use_signal};
use postcard::to_allocvec;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::ops::{Deref, DerefMut};
use std::sync::Mutex;
use std::hash::Hash;

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
    type Key: Eq + PartialEq + Hash + Clone + Debug;
    fn get_subscriptions() -> &'static Mutex<HashMap<Self::Key, StorageSender>>;
    fn subscribe<T: DeserializeOwned + 'static>(key: &Self::Key) -> Option<Receiver<StorageChannelPayload>>;
    fn get<T: DeserializeOwned>(key: &Self::Key) -> Option<T>;
    fn set<T: Serialize>(key: Self::Key, value: &T);
}

pub(crate) fn do_storage_backing_subscribe<S: StorageBacking + ?Sized + 'static, T: 'static>(key: &S::Key) -> Option<Receiver<StorageChannelPayload>> {
    log::info!("Subscribing to storage entry: {:?}", key);
    #[cfg(not(feature = "ssr"))]
    {
        let mut subscriptions = S::get_subscriptions().lock().unwrap();
        if let Some(channel) = subscriptions.get(key) {
            if channel.data_type_id == TypeId::of::<T>() {
                log::info!("Already subscribed to storage entry: {:?}", key);
                Some(channel.channel.new_receiver())
            } else {
                log::info!("Already subscribed to storage entry: {:?} but with a different type", key);
                None
            }
        } else {
            let (tx, rx) = broadcast::<StorageChannelPayload>(2);
            subscriptions.insert(
                key.clone(),
                StorageSender {
                    channel: tx,
                    data_type_id: TypeId::of::<T>(),
                },
            );
            log::info!("Subscribed to storage entry: {:?}", key);
            Some(rx)
        }
    }
    #[cfg(feature = "ssr")]
    {
        log::info!("Subscription not supported for SSR");
        None
    }
}


pub struct StorageSender {
    pub(crate) channel: Sender<StorageChannelPayload>,
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
    pub fn new(key: S::Key, data: T, subscribe: bool) -> Self {
        let channel = {
            #[cfg(feature = "ssr")]
            {
                StorageEntryChannel::default()
            }
            #[cfg(not(feature = "ssr"))]
            {
                let mut channel = StorageEntryChannel::new(S::subscribe::<T>(&key));
                if !subscribe {
                    channel.deactivate();
                }
                channel
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
    Active(Receiver<StorageChannelPayload>),
    Inactive(InactiveReceiver<StorageChannelPayload>),
}

impl StorageEntryChannel {
    fn new(receiver: Option<Receiver<StorageChannelPayload>>) -> Self {
        match receiver {
            Some(receiver) => Self::Active(receiver),
            None => Self::None,
        }
    }
    fn activate(&mut self) {
        if let Self::Inactive(channel) = self {
            *self = Self::Active(channel.activate_cloned())
        }
    }
    fn deactivate(&mut self) {
        if let Self::Active(channel) = self {
            *self = Self::Inactive(channel.clone().deactivate())
        }
    }
    pub async fn start_receiver_loop(&mut self, action:impl Fn(StorageChannelPayload)) {
        async fn receiver_loop(receiver: &mut Receiver<StorageChannelPayload>, action:impl Fn(StorageChannelPayload)) {
            loop {
                if let Ok(data) = receiver.recv().await {
                    action(data);
                }
            }
        }
        match self {
            Self::Active(receiver) => {
                receiver_loop(receiver, action).await;
            }
            Self::Inactive(_) => {
                self.activate();
                if let Self::Active(receiver) = self {
                    receiver_loop(receiver, action).await;
                }
            }
            _ => {}
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

pub fn synced_storage_entry<S, T>(key: S::Key, init: impl FnOnce() -> T, subscribe: bool) -> StorageEntry<S, T>
where
    S: StorageBacking,
    T: Serialize + DeserializeOwned + Clone + 'static,
    S::Key: Clone,
{
    let data = storage_entry::<S, T>(key.clone(), init);

    StorageEntry::new(key, data, subscribe)
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
        synced_storage_entry::<S, T>(key, init, true)
    });
    use_future!(cx, || async move {
        let mut channel = state.read().channel.clone();
        channel.start_receiver_loop(|_| state.with_mut(|storage_entry| storage_entry.update())).await;
    });
    state
}
