#![allow(unused)]
use dioxus::prelude::*;
use once_cell::sync::{Lazy, OnceCell};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::cell::{Ref, RefMut};
use std::fmt::Debug;
use std::io::Write;
use std::thread::LocalKey;
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};
use web_sys::{window, Storage};

use crate::storage::storage::{
    serde_from_string, serde_to_string, storage_entry, try_serde_from_string,
    use_synced_storage_entry, StorageBacking, StorageEntry, StorageEntryMut,
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

    fn get_subscriptions() -> &'static Mutex<HashMap<String, Box<dyn Any + Send>>> {
        &STORAGE_SUBSCRIPTIONS
    }

    fn subscribe<T: DeserializeOwned + Clone + Send>(key: &Self::Key) -> Option<Receiver<T>> {
        do_storage_backing_subscribe::<Self, T>(key)
    }

    fn set<T: Serialize>(key: String, value: &T) {
        set(key, value);
    }

    fn get<T: DeserializeOwned>(key: &String) -> Option<T> {
        get(key)
    }
}

static STORAGE_SUBSCRIPTIONS: Lazy<Mutex<HashMap<String, Box<dyn Any + Send>>>> = Lazy::new(|| {
    window()
        .unwrap()
        .add_event_listener_with_callback("storage", |e: web_sys::StorageEvent| {
            process_storage_event(e);
        })
        .unwrap();
    Mutex::new(HashMap::new())
});

fn process_storage_event(e: web_sys::StorageEvent) {
    let key = e.key().unwrap();
    let s: String = local_storage()?.get_item(&key).ok()??;
    let value = try_serde_from_string(&s)?;
    for subscription in STORAGE_SUBSCRIPTIONS.iter() {
        if subscription.key == key {
            subscription.callback(value);
        }
    }
}