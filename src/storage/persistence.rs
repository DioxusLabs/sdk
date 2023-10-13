use crate::storage::{
    storage_entry::{storage_entry, StorageEntry},
    SessionStorage,
};
use dioxus::prelude::*;
use dioxus_signals::use_signal;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// A persistent storage hook that can be used to store data across application reloads.
///
/// Depending on the platform this uses either local storage or a file storage
#[allow(clippy::needless_return)]
pub fn use_persistent<T: Serialize + DeserializeOwned + Default + Clone + 'static>(
    cx: &ScopeState,
    key: impl ToString,
    init: impl FnOnce() -> T,
) -> &mut StorageEntry<SessionStorage, T> {
    let mut init = Some(init);
    let state = {
        #[cfg(feature = "ssr")]
        {
            use_ref(cx, || {
                StorageEntry::<SessionStorage, T>::new(
                    key.to_string(),
                    init.take().unwrap()(),
                    None,
                )
            })
        }
        #[cfg(all(not(feature = "ssr"), not(feature = "hydrate")))]
        {
            cx.use_hook(|| {
                StorageEntry::<SessionStorage, T>::new(
                    key.to_string(),
                    storage_entry::<SessionStorage, T>(key.to_string(), init.take().unwrap()),
                    cx
                )
            })
        }
        #[cfg(all(not(feature = "ssr"), feature = "hydrate"))]
        {
            let state = use_ref(cx, || {
                StorageEntry::<ClientStorage, T>::new(key.to_string(), init.take().unwrap()(), None)
            });
            if cx.generation() == 0 {
                cx.needs_update();
            }
            if cx.generation() == 1 {
                state.set(StorageEntry::new(
                    key.to_string(),
                    storage_entry::<ClientStorage, T>(key.to_string(), init.take().unwrap()),
                ));
            }

            state
        }
    };
    state
}

/// A persistent storage hook that can be used to store data across application reloads.
/// The state will be the same for every call to this hook from the same line of code.
///
/// Depending on the platform this uses either local storage or a file storage
#[allow(clippy::needless_return)]
#[track_caller]
pub fn use_singleton_persistent<T: Serialize + DeserializeOwned + Default + Clone + 'static>(
    cx: &ScopeState,
    init: impl FnOnce() -> T,
) -> &mut StorageEntry<SessionStorage, T> {
    let caller = std::panic::Location::caller();
    let key = cx.use_hook(move || format!("{}:{}", caller.file(), caller.line()));
    log::info!("key: \"{}\"", key);
    use_persistent(cx, key, init)
}
