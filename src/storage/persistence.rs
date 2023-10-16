use crate::storage::{use_storage_entry, SessionStorage};
use dioxus::prelude::ScopeState;
use dioxus_signals::Signal;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// A persistent storage hook that can be used to store data across application reloads.
///
/// Depending on the platform this uses either local storage or a file storage
#[allow(clippy::needless_return)]
pub fn use_persistent<T: Serialize + DeserializeOwned + Default + Clone + PartialEq + 'static>(
    cx: &ScopeState,
    key: impl ToString,
    init: impl FnOnce() -> T,
) -> Signal<T> {
    let storage_entry = use_storage_entry::<SessionStorage, T>(cx, key.to_string(), init);
    storage_entry.data
}

/// A persistent storage hook that can be used to store data across application reloads.
/// The state will be the same for every call to this hook from the same line of code.
///
/// Depending on the platform this uses either local storage or a file storage
#[allow(clippy::needless_return)]
#[track_caller]
pub fn use_singleton_persistent<
    T: Serialize + DeserializeOwned + Default + Clone + PartialEq + 'static,
>(
    cx: &ScopeState,
    init: impl FnOnce() -> T,
) -> Signal<T> {
    let caller = std::panic::Location::caller();
    let key = cx.use_hook(move || format!("{}:{}", caller.file(), caller.line()));
    log::info!("use_singleton_persistent key: \"{}\"", key);
    use_persistent(cx, key, init)
}
