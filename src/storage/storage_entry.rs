use dioxus::prelude::ScopeState;
use dioxus_signals::{use_selector, use_signal, Signal};
use postcard::to_allocvec;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::{Debug, Display};
use std::ops::Deref;
use std::sync::{Arc, Mutex};

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
) -> &mut Signal<T>
where
    S: StorageBacking + StorageSubscriber<S>,
    T: Serialize + DeserializeOwned + Clone + 'static,
    S::Key: Clone,
{
    let key_signal = use_signal(cx, || key.clone());
    let state = cx.use_hook(|| synced_storage_entry::<S, T>(key, init, cx));
    let state_signal = state.data;
    if let Some(channel) = state.channel.clone() {
        use_listen_channel(cx, &channel, move |message| async move {
            if let Ok(payload) = message {
                *state_signal.write() = S::get(&key_signal.read()).unwrap_or_else(|| {
                    log::info!("get failed");
                    state_signal.read().clone()
                });
            }
        });
    }
    let state_clone = state.clone();
    use_selector(cx, move || {
        log::info!("use_synced_storage_entry selector");
        let x = state_signal;
        state_clone.save();
    });
    &mut state.data
}
