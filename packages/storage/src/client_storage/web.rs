use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use dioxus::logger::tracing::{error, trace};
use once_cell::sync::Lazy;
use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::watch::{Receiver, channel};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::{Storage, window};

use crate::{
    StorageBacking, StorageChannelPayload, StorageSubscriber, StorageSubscription, serde_to_string,
    try_serde_from_string,
};

#[derive(Clone)]
pub struct LocalStorage;

impl StorageBacking for LocalStorage {
    type Key = String;

    fn set<T: Serialize + Send + Sync + 'static>(key: String, value: &T) {
        set(key, value, WebStorageType::Local);
    }

    fn get<T: DeserializeOwned>(key: &String) -> Option<T> {
        get(key, WebStorageType::Local)
    }
}

impl StorageSubscriber<LocalStorage> for LocalStorage {
    fn subscribe<T: DeserializeOwned + Send + Sync + Clone + 'static>(
        key: &String,
    ) -> Receiver<StorageChannelPayload> {
        let read_binding = SUBSCRIPTIONS.read().unwrap();
        match read_binding.get(key) {
            Some(subscription) => subscription.tx.subscribe(),
            None => {
                drop(read_binding);
                let (tx, rx) = channel::<StorageChannelPayload>(StorageChannelPayload::default());
                let subscription = StorageSubscription::new::<LocalStorage, T>(tx, key.clone());
                SUBSCRIPTIONS
                    .write()
                    .unwrap()
                    .insert(key.clone(), subscription);
                rx
            }
        }
    }

    fn unsubscribe(key: &String) {
        let read_binding = SUBSCRIPTIONS.read().unwrap();
        if let Some(entry) = read_binding.get(key) {
            if entry.tx.is_closed() {
                drop(read_binding);
                SUBSCRIPTIONS.write().unwrap().remove(key);
            }
        }
    }
}

/// A map of all the channels that are currently subscribed to and the getters for the corresponding storage entry. This gets initialized lazily and will set up a listener for storage events.
static SUBSCRIPTIONS: Lazy<Arc<RwLock<HashMap<String, StorageSubscription>>>> = Lazy::new(|| {
    // Create a closure that will be called when a storage event occurs.
    let closure = Closure::wrap(Box::new(move |e: web_sys::StorageEvent| {
        trace!("Storage event: {:?}", e);
        let key: String = e.key().unwrap();
        let read_binding = SUBSCRIPTIONS.read().unwrap();
        if let Some(subscription) = read_binding.get(&key) {
            if subscription.tx.is_closed() {
                trace!("Channel is closed, removing subscription for \"{}\"", key);
                drop(read_binding);
                SUBSCRIPTIONS.write().unwrap().remove(&key);
                return;
            }
            // Call the getter for the given entry and send the value to said entry's channel.
            match subscription.get_and_send() {
                Ok(_) => trace!("Sent storage event"),
                Err(err) => error!("Error sending storage event: {:?}", err.to_string()),
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

#[derive(Clone)]
pub struct SessionStorage;

impl StorageBacking for SessionStorage {
    type Key = String;

    fn set<T: Serialize + Send + Sync + 'static>(key: String, value: &T) {
        set(key, value, WebStorageType::Session);
    }

    fn get<T: DeserializeOwned>(key: &String) -> Option<T> {
        get(key, WebStorageType::Session)
    }
}

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
