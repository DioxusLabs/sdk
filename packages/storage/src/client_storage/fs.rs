use crate::{StorageChannelPayload, StorageSubscription};
use dioxus::logger::tracing::trace;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::io::Write;
use std::sync::{OnceLock, RwLock};
use tokio::sync::watch::{Receiver, channel};

use crate::{StorageBacking, StoragePersistence, StorageSubscriber};

#[doc(hidden)]
/// Sets the directory where the storage files are located.
pub fn set_directory(path: std::path::PathBuf) {
    LOCATION.set(path).unwrap();
}

#[doc(hidden)]
pub fn set_dir_name(name: &str) {
    set_directory(
        directories::BaseDirs::new()
            .unwrap()
            .data_local_dir()
            .join(name),
    )
}

/// The location where the storage files are located.
static LOCATION: OnceLock<std::path::PathBuf> = OnceLock::new();

/// Set a value in the configured storage location using the key as the file name.
fn set(key: &str, as_str: &Option<String>) {
    let path = LOCATION
        .get()
        .expect("Call the set_dir macro before accessing persistent data");

    let file_path = path.join(key);

    match as_str {
        Some(as_str) => {
            std::fs::create_dir_all(path).unwrap();
            let mut file = std::fs::File::create(file_path).unwrap();
            file.write_all(as_str.as_bytes()).unwrap()
        }
        None => match std::fs::remove_file(file_path) {
            Ok(_) => {}
            Err(error) => match error.kind() {
                std::io::ErrorKind::NotFound => {}
                _ => Result::Err(error).unwrap(),
            },
        },
    }
}

/// Get a value from the configured storage location using the key as the file name.
fn get(key: &str) -> Option<String> {
    let path = LOCATION
        .get()
        .expect("Call the set_dir macro before accessing persistent data")
        .join(key);
    std::fs::read_to_string(path).ok()
}

#[derive(Clone)]
pub struct LocalStorage;

/// LocalStorage stores Option<String>.
impl StoragePersistence for LocalStorage {
    type Key = String;
    type Value = Option<String>;

    fn load(key: &Self::Key) -> Self::Value {
        get(key)
    }

    fn store(key: Self::Key, value: &Self::Value) {
        set(&key, value);

        // If the subscriptions map is not initialized, we don't need to notify any subscribers.
        if let Some(subscriptions) = SUBSCRIPTIONS.get() {
            let read_binding = subscriptions.read().unwrap();
            if let Some(subscription) = read_binding.get(&key) {
                subscription
                    .tx
                    .send(StorageChannelPayload::new(value.clone()))
                    .unwrap();
            }
        }
    }
}

// Note that this module contains an optimization that differs from the web version. Dioxus Desktop runs all windows in
// the same thread, meaning that we can just directly notify the subscribers via the same channels, rather than using the
// storage event listener.
impl StorageSubscriber<LocalStorage> for LocalStorage {
    fn subscribe<T: DeserializeOwned + Send + Sync + Clone + 'static>(
        key: &<LocalStorage as StorageBacking>::Key,
    ) -> Receiver<StorageChannelPayload> {
        // Initialize the subscriptions map if it hasn't been initialized yet.
        let subscriptions = SUBSCRIPTIONS.get_or_init(|| RwLock::new(HashMap::new()));

        // Check if the subscription already exists. If it does, return the existing subscription's channel.
        // If it doesn't, create a new subscription and return its channel.
        let read_binding = subscriptions.read().unwrap();
        match read_binding.get(key) {
            Some(subscription) => subscription.tx.subscribe(),
            None => {
                drop(read_binding);
                let (tx, rx) = channel::<StorageChannelPayload>(StorageChannelPayload::default());
                let subscription = StorageSubscription::new::<LocalStorage, T>(tx, key.clone());

                subscriptions
                    .write()
                    .unwrap()
                    .insert(key.clone(), subscription);
                rx
            }
        }
    }

    fn unsubscribe(key: &<LocalStorage as StorageBacking>::Key) {
        trace!("Unsubscribing from \"{}\"", key);

        // Fail silently if unsubscribe is called but the subscriptions map isn't initialized yet.
        if let Some(subscriptions) = SUBSCRIPTIONS.get() {
            let read_binding = subscriptions.read().unwrap();

            // If the subscription exists, remove it from the subscriptions map.
            if read_binding.contains_key(key) {
                trace!("Found entry for \"{}\"", key);
                drop(read_binding);
                subscriptions.write().unwrap().remove(key);
            }
        }
    }
}

/// A map of all the channels that are currently subscribed to and the getters for the corresponding storage entry.
/// This gets initialized lazily.
static SUBSCRIPTIONS: OnceLock<RwLock<HashMap<String, StorageSubscription>>> = OnceLock::new();
