use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use dioxus::prelude::*;
use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::watch::{channel, Receiver};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::{window, Storage};

use crate::storage::{
    serde_to_string, try_serde_from_string, StorageBacking, StorageChannelPayload,
    StorageEventChannel, StorageSubscriber,
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
        let read_binding = CHANNELS.read().unwrap();
        match read_binding.get(key) {
            Some(entry) => entry.tx.subscribe(),
            None => {
                drop(read_binding);
                let (tx, rx) = channel::<StorageChannelPayload>(StorageChannelPayload::default());
                let entry = StorageEventChannel::new::<LocalStorage, T>(tx, key.clone());
                CHANNELS.write().unwrap().insert(key.clone(), entry);
                rx
            }
        }
    }

    fn unsubscribe(key: &String) {
        log::info!("Unsubscribing from \"{}\"", key);
        let read_binding = CHANNELS.read().unwrap();
        if let Some(entry) = read_binding.get(key) {
            log::info!("Found entry for \"{}\"", key);
            if entry.tx.is_closed() {
                log::info!("Channel is closed, removing entry for \"{}\"", key);
                drop(read_binding);
                CHANNELS.write().unwrap().remove(key);
            }
        }
    }
}

/// A map of all the channels that are currently subscribed to. This gets initialized lazily and will set up a listener for storage events.
static CHANNELS: Lazy<Arc<RwLock<HashMap<String, StorageEventChannel>>>> = Lazy::new(|| {
    // Create a closure that will be called when a storage event occurs.
    let closure = Closure::wrap(Box::new(move |e: web_sys::StorageEvent| {
        log::info!("Storage event: {:?}", e);
        let key: String = e.key().unwrap();
        let read_binding = CHANNELS.read().unwrap();
        if let Some(entry) = read_binding.get(&key) {
            if entry.tx.is_closed() {
                log::info!("Channel is closed, removing entry for \"{}\"", key);
                drop(read_binding);
                CHANNELS.write().unwrap().remove(&key);
                return;
            }
            // Call the getter for the given entry and send the value to said entry's channel.
            match entry.get_and_send() {
                Ok(_) => log::info!("Sent storage event"),
                Err(err) => log::error!("Error sending storage event: {:?}", err.to_string()),
            }
        }
    }) as Box<dyn FnMut(web_sys::StorageEvent)>);
    // Register the closure to be called when a storage event occurs.
    window()
        .unwrap()
        .add_event_listener_with_callback("storage", closure.as_ref().unchecked_ref())
        .unwrap();
    // Relinquish ownership of the closure to the JS runtime so that it can be called later.
    closure.forget();
    Arc::new(RwLock::new(HashMap::new()))
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
