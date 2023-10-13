use async_broadcast::{broadcast, Receiver, InactiveReceiver, Sender};
use dioxus::prelude::*;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::OnceLock;
use uuid::Uuid;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::{window, Storage};

use crate::storage::storage_entry::{
    serde_to_string, try_serde_from_string, StorageBacking, StorageChannelPayload,
    StorageSubscriber,
};
use crate::utils::channel::UseChannel;

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
    fn subscribe<T: DeserializeOwned + 'static>(
        _cx: &ScopeState,
        _key: &String,
    ) -> Option<Receiver<StorageChannelPayload<Self>>> {
        let (_, rx) = CHANNEL.get_or_init(|| {
            let (tx, rx) = broadcast::<StorageChannelPayload<Self>>(5);
            let tx_clone = tx.clone();
            let closure = Closure::wrap(Box::new(move |e: web_sys::StorageEvent| {
                log::info!("Storage event: {:?}", e);
                let key: String = e.key().unwrap();
                let tx_clone_clone  = tx_clone.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let result = tx_clone_clone
                        .broadcast(StorageChannelPayload::<Self> { key })
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
            (tx, rx.deactivate())
        });
        Some(rx.activate_cloned())
    }

    fn unsubscribe(_key: &String) {
        // Do nothing for web case, since we don't actually subscribe to specific keys.
    }
}

static CHANNEL: OnceLock<(Sender<StorageChannelPayload<LocalStorage>>, InactiveReceiver<StorageChannelPayload<LocalStorage>>)> = OnceLock::new();
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
    #[cfg(not(feature = "ssr"))]
    {
        let as_str = serde_to_string(value);
        get_storage_by_type(storage_type)
            .unwrap()
            .set_item(&key, &as_str)
            .unwrap();
    }
}

fn get<T: DeserializeOwned>(key: &str, storage_type: WebStorageType) -> Option<T> {
    #[cfg(not(feature = "ssr"))]
    {
        let s: String = get_storage_by_type(storage_type)?.get_item(key).ok()??;
        try_serde_from_string(&s)
    }
    #[cfg(feature = "ssr")]
    None
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
