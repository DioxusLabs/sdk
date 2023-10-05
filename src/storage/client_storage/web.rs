#![allow(unused)]
use async_broadcast::Receiver;
use dioxus::prelude::*;
use once_cell::sync::{Lazy, OnceCell};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use std::any::TypeId;
use std::cell::{Ref, RefMut};
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Write;
use std::sync::Mutex;
use std::thread::LocalKey;
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};
use web_sys::{window, Storage};

use crate::storage::storage::{
    serde_from_string, serde_to_string, storage_entry, try_serde_from_string,
    use_synced_storage_entry, StorageBacking, StorageEntry, StorageEntryMut, StorageSender, do_storage_backing_subscribe, StorageChannelPayload,
};

fn local_storage() -> Option<Storage> {
    window()?.local_storage().ok()?
}

fn set<T: Serialize>(key: String, value: &T) {
    #[cfg(not(feature = "ssr"))]
    {
        let as_str = serde_to_string(value);
        local_storage().unwrap().set_item(&key, &as_str).unwrap();
    }
}

fn get<T: DeserializeOwned>(key: &str) -> Option<T> {
    #[cfg(not(feature = "ssr"))]
    {
        let s: String = local_storage()?.get_item(key).ok()??;
        try_serde_from_string(&s)
    }
    #[cfg(feature = "ssr")]
    None
}

pub struct ClientStorage;

impl StorageBacking for ClientStorage {
    type Key = String;

    fn get_subscriptions() -> &'static Mutex<HashMap<String, StorageSender>> {
        &STORAGE_SUBSCRIPTIONS
    }

    fn subscribe<T: DeserializeOwned + 'static>(key: &Self::Key) -> Option<Receiver<StorageChannelPayload>> {
        do_storage_backing_subscribe::<Self, T>(key)
    }

    fn set<T: Serialize>(key: String, value: &T) {
        set(key, value);
    }

    fn get<T: DeserializeOwned>(key: &String) -> Option<T> {
        get(key)
    }
}

static STORAGE_SUBSCRIPTIONS: Lazy<Mutex<HashMap<String, StorageSender>>> = Lazy::new(|| {

    log::info!("Initializing storage subscriptions");
    let closure = Closure::<dyn FnMut(web_sys::StorageEvent)>::wrap(Box::new(|e: web_sys::StorageEvent| {
        process_storage_event(e);
    }));
    window()
        .unwrap()
        .add_event_listener_with_callback("storage", closure.as_ref().unchecked_ref())
        .unwrap();
    Mutex::new(HashMap::new())
});

fn process_storage_event(e: web_sys::StorageEvent) {
    log::info!("Incoming storage event");
    let key = e.key().unwrap();
    if let Some(storage) = local_storage() {
        let value = storage.get_item(&key).unwrap();
        for subscription in STORAGE_SUBSCRIPTIONS.lock().unwrap().values() {
            if subscription.data_type_id == TypeId::of::<String>() {
                log::info!("broadcasting");
                subscription.channel.broadcast(StorageChannelPayload::Updated);
            }
        }
    }
    // let s: String = local_storage().map(f)?.get_item(&key).ok()??;
    // let value = try_serde_from_string(&s)?;
    // for subscription in STORAGE_SUBSCRIPTIONS.iter() {
    //     if subscription.key == key {
    //         subscription.callback(value);
    //     }
    // }
}