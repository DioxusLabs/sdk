use crate::storage::{StorageChannelPayload, StorageSubscription};
use dioxus::prelude::*;
use notify::Watcher;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::io::Write;
use std::sync::{OnceLock, RwLock};
use tokio::sync::{mpsc, watch};

use crate::storage::{serde_to_string, try_serde_from_string, StorageBacking, StorageSubscriber};

#[allow(clippy::needless_doctest_main)]
/// Set the directory where the storage files are located on non-wasm targets.
///
/// ```rust
/// fn main(){
///     // set the directory to the default location
///     set_dir!();
///     // set the directory to a custom location
///     set_dir!(PathBuf::from("path/to/dir"));
/// }
/// ```
#[macro_export]
macro_rules! set_dir {
    () => {
        extern crate self as storage;
        storage::set_dir_name(env!("CARGO_PKG_NAME"));
    };
    ($path: literal) => {
        extern crate self as storage;
        storage::set_directory(std::path::PathBuf::from($path));
    };
}
pub use set_dir;

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
fn set<T: Serialize>(key: String, value: &T) {
    let as_str = serde_to_string(value);
    let path = LOCATION
        .get()
        .expect("Call the set_dir macro before accessing persistant data");
    std::fs::create_dir_all(path).unwrap();
    let file_path = path.join(key);
    let mut file = std::fs::File::create(file_path).unwrap();
    file.write_all(as_str.as_bytes()).unwrap();
}

/// Get a value from the configured storage location using the key as the file name.
fn get<T: DeserializeOwned>(key: &str) -> Option<T> {
    let path = LOCATION
        .get()
        .expect("Call the set_dir macro before accessing persistant data")
        .join(key);
    let s = std::fs::read_to_string(path).ok()?;
    try_serde_from_string(&s)
}

#[derive(Clone)]
pub struct LocalStorage;

impl StorageBacking for LocalStorage {
    type Key = String;

    fn set<T: Serialize>(key: String, value: &T) {
        set(key, value);
    }

    fn get<T: DeserializeOwned>(key: &String) -> Option<T> {
        get(key)
    }
}

impl StorageSubscriber<LocalStorage> for LocalStorage {
    fn subscribe<T: DeserializeOwned + Send + Sync + 'static>(
        cx: &ScopeState,
        key: &<LocalStorage as StorageBacking>::Key,
    ) -> watch::Receiver<StorageChannelPayload> {
        // Initialize the watcher helper if it hasn't been initialized yet.
        let watcher_helper = WATCHER_HELPER.get_or_init(|| {
            let (tx, mut rx) = mpsc::channel::<WatcherAction>(10);

            // The watcher is spawned and managed in a separate thread because it is not thread-safe.
            cx.spawn_forever(async move {
                // Create a watcher and set up a listener for storage events that will notify the correct subscribers.
                let mut watcher =
                    notify::recommended_watcher(|result: Result<notify::Event, notify::Error>| {
                        match result {
                            Ok(event) => {
                                // Get the path of the file that was changed and use it as the key.
                                let path = event.paths.first().unwrap();
                                let key = path.file_name().unwrap().to_str().unwrap().to_string();
                                let read_binding =
                                    WATCHER_HELPER.get().unwrap().subscriptions.read().unwrap();
                                if let Some(subscription) = read_binding.get(&key) {
                                    // If the subscription channel is closed, remove it from the subscriptions map the watcher.
                                    if subscription.tx.is_closed() {
                                        log::info!(
                                            "Channel is closed, removing subscription for \"{}\"",
                                            key
                                        );
                                        drop(read_binding);
                                        WATCHER_HELPER
                                            .get()
                                            .unwrap()
                                            .subscriptions
                                            .write()
                                            .unwrap()
                                            .remove(&key);
                                        return;
                                    }
                                    // Call the getter for the given entry and send the value to said entry's channel.
                                    match subscription.get_and_send() {
                                        Ok(_) => log::info!("Sent storage event"),
                                        Err(err) => log::error!(
                                            "Error sending storage event: {:?}",
                                            err.to_string()
                                        ),
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Error watching file: {}", e);
                            }
                        }
                    })
                    .unwrap();
                // Create an infinite loop that will listen for watcher actions and subscribe/unsubscribe from the watcher accordingly.
                while let Some(message) = rx.recv().await {
                    match message {
                        WatcherAction::Subscribe(key) => {
                            let path = LOCATION
                                .get()
                                .expect("Call the set_dir macro before accessing persistant data")
                                .join(key);
                            watcher
                                .watch(&path, notify::RecursiveMode::NonRecursive)
                                .unwrap();
                        }
                        WatcherAction::Unsubscribe(key) => {
                            let path = LOCATION
                                .get()
                                .expect("Call the set_dir macro before accessing persistant data")
                                .join(key);
                            watcher.unwatch(&path).unwrap();
                        }
                    }
                }
            });
            WatcherHelper {
                channel: tx,
                subscriptions: RwLock::new(HashMap::new()),
            }
        });

        // Check if the subscription already exists. If it does, return the existing subscription's channel.
        // If it doesn't, create a new subscription, register it with the watcher, and return its channel.
        let read_binding = watcher_helper.subscriptions.read().unwrap();
        match read_binding.get(key) {
            Some(subscription) => subscription.tx.subscribe(),
            None => {
                drop(read_binding);
                let (tx, rx) =
                    watch::channel::<StorageChannelPayload>(StorageChannelPayload::default());
                let subscription = StorageSubscription::new::<LocalStorage, T>(tx, key.clone());

                watcher_helper
                    .subscriptions
                    .write()
                    .unwrap()
                    .insert(key.clone(), subscription);
                watcher_helper
                    .channel
                    .try_send(WatcherAction::Subscribe(key.clone()))
                    .unwrap();
                rx
            }
        }
    }

    fn unsubscribe(key: &<LocalStorage as StorageBacking>::Key) {
        log::info!("Unsubscribing from \"{}\"", key);

        // Fail silently if unsubscribe is called but the watcher isn't initialized yet.
        if let Some(watcher_helper) = WATCHER_HELPER.get() {
            let read_binding = watcher_helper.subscriptions.read().unwrap();

            // If the subscription exists, remove it from the subscriptions map and send an unsubscribe message to the watcher.
            if let Some(entry) = read_binding.get(key) {
                log::info!("Found entry for \"{}\"", key);
                drop(read_binding);
                watcher_helper
                    .channel
                    .try_send(WatcherAction::Unsubscribe(key.clone()))
                    .unwrap();
                watcher_helper.subscriptions.write().unwrap().remove(key);
            }
        }
    }
}

/// A map of all the channels that are currently subscribed to and the getters for the corresponding storage entry.
/// This gets initialized lazily and will set up a listener for storage events.
static WATCHER_HELPER: OnceLock<WatcherHelper> = OnceLock::new();

/// A helper struct that manages the watcher channel and subscriptions.
struct WatcherHelper {
    /// The channel that the application uses to communicate with the watcher thread.
    channel: mpsc::Sender<WatcherAction>,
    /// A map of all the subscriptions that are currently active.
    subscriptions: RwLock<HashMap<String, StorageSubscription>>,
}

/// The actions that can be sent to the watcher thread.
enum WatcherAction {
    /// Subscribe to a key.
    Subscribe(String),
    /// Unsubscribe from a key.
    Unsubscribe(String),
}
