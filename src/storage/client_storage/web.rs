use async_broadcast::broadcast;
use dioxus::prelude::*;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::OnceLock;
use uuid::Uuid;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::{window, Storage};

use crate::storage::storage::{
    serde_to_string, try_serde_from_string,
    StorageBacking, StorageChannelPayload, LocalStorageBacking, StorageType,
};
use crate::utils::channel::UseChannel;

// Start LocalStorage
#[derive(Clone)]
pub struct LocalStorage;

impl StorageBacking for LocalStorage {
    type Key = String;
    type Local = Self;

    fn set<T: Serialize>(key: String, value: &T) {
        set(key, value, StorageType::Local);
    }

    fn get<T: DeserializeOwned>(key: &String) -> Option<T> {
        get(key, StorageType::Local)
    }

    fn is_local_storage() -> bool {
        true
    }
}

impl LocalStorageBacking for LocalStorage {

    fn subscribe<T: DeserializeOwned + 'static>(
        _cx: &ScopeState,
        _key: &Self::Key,
    ) -> Option<UseChannel<StorageChannelPayload<Self>>> {
        let channel = CHANNEL.get_or_init(|| {
            let (tx, rx) = broadcast::<StorageChannelPayload<Self>>(5);
            let channel = UseChannel::new(Uuid::new_v4(), tx, rx.deactivate());
            let channel_clone = channel.clone();

            let closure = Closure::wrap(Box::new(move |e: web_sys::StorageEvent| {
                log::info!("Storage event: {:?}", e);
                let key: String = e.key().unwrap();
                let channel_clone_clone = channel_clone.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let result = channel_clone_clone
                        .send(StorageChannelPayload::<Self> { key })
                        .await;
                    match result {
                        Ok(_) => log::info!("Sent storage event"),
                        Err(err) => log::info!("Error sending storage event: {:?}", err),
                    }
                });
            }) as Box<dyn FnMut(web_sys::StorageEvent)>);
            window()
                .unwrap()
                .add_event_listener_with_callback("storage", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
            channel
        });
        Some(channel.clone())
    }

    fn unsubscribe(_key: &Self::Key) {
        // Do nothing for web case, since we don't actually subscribe to specific keys.
    }
}

static CHANNEL: OnceLock<UseChannel<StorageChannelPayload<LocalStorage>>> = OnceLock::new();
// End LocalStorage

// Start SessionStorage
#[derive(Clone)]
pub struct SessionStorage;

impl StorageBacking for SessionStorage {
    type Key = String;
    // Ideally the following should be the experimental !(never) type, but that's not stable yet.
    type Local = LocalStorage;

    fn set<T: Serialize>(key: String, value: &T) {
        set(key, value, StorageType::Session);
    }

    fn get<T: DeserializeOwned>(key: &String) -> Option<T> {
        get(key, StorageType::Session)
    }

    fn is_local_storage() -> bool {
        false
    }
}
// End SessionStorage

// Start common
fn set<T: Serialize>(key: String, value: &T, storage_type: StorageType) {
    #[cfg(not(feature = "ssr"))]
    {
        let as_str = serde_to_string(value);
        get_storage_by_type(storage_type).unwrap().set_item(&key, &as_str).unwrap();
    }
}

fn get<T: DeserializeOwned>(key: &str, storage_type: StorageType) -> Option<T> {
    #[cfg(not(feature = "ssr"))]
    {
        let s: String = get_storage_by_type(storage_type)?.get_item(key).ok()??;
        try_serde_from_string(&s)
    }
    #[cfg(feature = "ssr")]
    None
}

fn get_storage_by_type(storage_type: StorageType) -> Option<Storage> {
    window().map_or_else(|| None, |window| {
        match storage_type {
            StorageType::Local => window.local_storage().ok()?,
            StorageType::Session => window.session_storage().ok()?,
        }
    })
}
// End common
