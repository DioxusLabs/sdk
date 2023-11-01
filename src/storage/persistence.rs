use crate::storage::storage_entry;
use crate::storage::SessionStorage;
use dioxus::prelude::ScopeState;
use dioxus_signals::Signal;
use serde::de::DeserializeOwned;
use serde::Serialize;

use super::StorageEntryTrait;

/// A persistent storage hook that can be used to store data across application reloads.
///
/// Depending on the platform this uses either local storage or a file storage
#[allow(clippy::needless_return)]
pub fn use_persistent<
    T: Serialize + DeserializeOwned + Default + Clone + Send + Sync + PartialEq + 'static,
>(
    cx: &ScopeState,
    key: impl ToString,
    init: impl FnOnce() -> T,
) -> Signal<T> {
    *cx.use_hook(|| persistent(key, init))
}

/// Creates a persistent storage signal that can be used to store data across application reloads.
///
/// Depending on the platform this uses either local storage or a file storage
#[allow(clippy::needless_return)]
pub fn persistent<
    T: Serialize + DeserializeOwned + Default + Clone + Send + Sync + PartialEq + 'static,
>(
    key: impl ToString,
    init: impl FnOnce() -> T,
) -> Signal<T> {
    let storage_entry = storage_entry::<SessionStorage, T>(key.to_string(), init);
    storage_entry.save_to_storage_on_change();
    storage_entry.data
}

/// A persistent storage hook that can be used to store data across application reloads.
/// The state will be the same for every call to this hook from the same line of code.
///
/// Depending on the platform this uses either local storage or a file storage
#[allow(clippy::needless_return)]
#[track_caller]
pub fn use_singleton_persistent<
    T: Serialize + DeserializeOwned + Default + Clone + Send + Sync + PartialEq + 'static,
>(
    cx: &ScopeState,
    init: impl FnOnce() -> T,
) -> Signal<T> {
    *cx.use_hook(|| singleton_persistent(init))
}

/// Create a persistent storage signal that can be used to store data across application reloads.
/// The state will be the same for every call to this hook from the same line of code.
///
/// Depending on the platform this uses either local storage or a file storage
#[allow(clippy::needless_return)]
#[track_caller]
pub fn singleton_persistent<
    T: Serialize + DeserializeOwned + Default + Clone + Send + Sync + PartialEq + 'static,
>(
    init: impl FnOnce() -> T,
) -> Signal<T> {
    let caller = std::panic::Location::caller();
    let key = format!("{}:{}", caller.file(), caller.line());
    log::trace!("singleton_persistent key: \"{}\"", key);
    persistent(key, init)
}
