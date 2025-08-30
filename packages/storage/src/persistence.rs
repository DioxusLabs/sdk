//! Storage utilities which implicitly use [LocalStorage].
//!
//! These do not sync: if another session writes to them it will not trigger an update.

use crate::LocalStorage;
use crate::{new_storage_entry, use_hydrate_storage};
use dioxus::prelude::*;
use dioxus_signals::Signal;
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::StorageEntryTrait;

/// What storage to use.
///
/// TODO:
/// Documentation on the APIs implies this just needs to live across reloads, which for web would be session storage, but for desktop would require local storage.
/// Since docs currently say "local storage" local storage is being used.
type Storage = LocalStorage;

/// A persistent storage hook that can be used to store data across application reloads.
///
/// Depending on the platform this uses either local storage or a file storage
pub fn use_persistent<
    T: Serialize + DeserializeOwned + Clone + Send + Sync + PartialEq + 'static,
>(
    key: impl ToString,
    init: impl FnOnce() -> T,
) -> Signal<T> {
    let mut init = Some(init);
    let storage = use_hook(|| new_persistent(key.to_string(), || init.take().unwrap()()));
    use_hydrate_storage::<Storage, T>(storage, init);
    storage
}

/// Creates a persistent storage signal that can be used to store data across application reloads.
///
/// Depending on the platform this uses either local storage or a file storage
pub fn new_persistent<
    T: Serialize + DeserializeOwned + Clone + Send + Sync + PartialEq + 'static,
>(
    key: impl ToString,
    init: impl FnOnce() -> T,
) -> Signal<T> {
    let storage_entry = new_storage_entry::<Storage, T>(key.to_string(), init);
    StorageEntryTrait::<Storage, T>::save_to_storage_on_change(&storage_entry);
    storage_entry.data
}

/// A persistent storage hook that can be used to store data across application reloads.
/// The state will be the same for every call to this hook from the same line of code.
///
/// Depending on the platform this uses either local storage or a file storage
#[track_caller]
pub fn use_singleton_persistent<
    T: Serialize + DeserializeOwned + Clone + Send + Sync + PartialEq + 'static,
>(
    init: impl FnOnce() -> T,
) -> Signal<T> {
    let mut init = Some(init);
    let signal = use_hook(|| new_singleton_persistent(|| init.take().unwrap()()));
    use_hydrate_storage::<Storage, T>(signal, init);
    signal
}

/// Create a persistent storage signal that can be used to store data across application reloads.
/// The state will be the same for every call to this hook from the same line of code.
///
/// Depending on the platform this uses either local storage or a file storage
#[allow(clippy::needless_return)]
#[track_caller]
pub fn new_singleton_persistent<
    T: Serialize + DeserializeOwned + Clone + Send + Sync + PartialEq + 'static,
>(
    init: impl FnOnce() -> T,
) -> Signal<T> {
    let caller = std::panic::Location::caller();
    let key = format!("{}:{}", caller.file(), caller.line());
    new_persistent(key, init)
}
