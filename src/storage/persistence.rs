use crate::storage::{SessionStorage, use_storage_entry};
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
) -> &mut Signal<T> {
    let mut init = Some(init);
    let storage_entry = {
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
            use_storage_entry::<SessionStorage,T>(cx, key.to_string(), init.take().unwrap())
        }
        #[cfg(all(not(feature = "ssr"), feature = "hydrate"))]
        {
            let state = cx.use_hook(|| {
                StorageEntry::<SessionStorage, T>::new(
                    key.to_string(),
                    storage_entry::<SessionStorage, T>(key.to_string(), init.take().unwrap()),
                    cx,
                )
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
    &mut storage_entry.data
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
) -> &mut Signal<T> {
    let caller = std::panic::Location::caller();
    let key = cx.use_hook(move || format!("{}:{}", caller.file(), caller.line()));
    log::info!("use_singleton_persistent key: \"{}\"", key);
    use_persistent(cx, key, init)
}
