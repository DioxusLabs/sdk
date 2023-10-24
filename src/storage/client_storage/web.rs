use dashmap::DashMap;
use dioxus::prelude::*;
use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::watch::{channel, Receiver};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::{window, Storage};

use crate::storage::{
    serde_to_string, try_serde_from_string, StorageBacking, StorageChannelPayload,
    StorageSenderEntry, StorageSubscriber,
};

// Start LocalStorage
#[derive(Clone)]
pub struct LocalStorage;

impl StorageBacking for LocalStorage {
    type Key = String;

    fn set<T: Serialize>(key: String, value: &T) {
        set(key, value, WebStorageType::Local);
    }

    fn get<T: DeserializeOwned>(key: &String) -> Option<T> {
        get(key, WebStorageType::Local)
    }
}

impl StorageSubscriber<LocalStorage> for LocalStorage {
    fn subscribe<T: DeserializeOwned + Send + Sync + 'static>(
        _cx: &ScopeState,
        key: &String,
    ) -> Receiver<StorageChannelPayload> {
        let rx = CHANNELS.get(key).map_or_else(
            || {
                let (tx, rx) = channel::<StorageChannelPayload>(StorageChannelPayload::default());
                let entry = StorageSenderEntry::new::<LocalStorage, T>(tx, key.clone());
                CHANNELS.insert(key.clone(), entry);
                rx
            },
            |entry| entry.tx.subscribe(),
        );
        rx
    }

    fn unsubscribe(key: &String) {
        if let Some(entry) = CHANNELS.get(key) {
            if entry.tx.is_closed() {
                CHANNELS.remove(key);
            }
        }
    }
}

static CHANNELS: Lazy<DashMap<String, StorageSenderEntry>> = Lazy::new(|| {
    let closure = Closure::wrap(Box::new(move |e: web_sys::StorageEvent| {
        log::info!("Storage event: {:?}", e);
        let key: String = e.key().unwrap();
        if let Some(entry) = CHANNELS.get(&key) {
            let result = entry.tx.send((entry.getter)());
            match result {
                Ok(_) => log::info!("Sent storage event"),
                Err(err) => log::info!("Error sending storage event: {:?}", err),
            }
        }
    }) as Box<dyn FnMut(web_sys::StorageEvent)>);
    window()
        .unwrap()
        .add_event_listener_with_callback("storage", closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();
    DashMap::new()
});

// End LocalStorage

// Start SessionStorage
#[derive(Clone)]
pub struct SessionStorage;

impl StorageBacking for SessionStorage {
    type Key = String;

    fn set<T: Serialize>(key: String, value: &T) {
        set(key, value, WebStorageType::Session);
    }

    fn get<T: DeserializeOwned>(key: &String) -> Option<T> {
        get(key, WebStorageType::Session)
    }
}
// End SessionStorage

// Start common
fn set<T: Serialize>(key: String, value: &T, storage_type: WebStorageType) {
    let as_str = serde_to_string(value);
    get_storage_by_type(storage_type)
        .unwrap()
        .set_item(&key, &as_str)
        .unwrap();
}

fn get<T: DeserializeOwned>(key: &str, storage_type: WebStorageType) -> Option<T> {
    let s: String = get_storage_by_type(storage_type)?.get_item(key).ok()??;
    try_serde_from_string(&s)
}

fn get_storage_by_type(storage_type: WebStorageType) -> Option<Storage> {
    window().map_or_else(
        || None,
        |window| match storage_type {
            WebStorageType::Local => window.local_storage().ok()?,
            WebStorageType::Session => window.session_storage().ok()?,
        },
    )
}

enum WebStorageType {
    Local,
    Session,
}
// End common
