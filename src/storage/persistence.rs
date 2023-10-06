use dioxus::prelude::*;
use dioxus_signals::{use_signal, Signal, Write};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::cell::Ref;
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};
use crate::storage::storage::StorageEntryChannel;
use crate::utils::channel::use_listen_channel;
use crate::storage::{
    ClientStorage,
    storage::{
        storage_entry, StorageEntry,
    },
};

/// A persistent storage hook that can be used to store data across application reloads.
///
/// Depending on the platform this uses either local storage or a file storage
#[allow(clippy::needless_return)]
pub fn use_persistent<T: Serialize + DeserializeOwned + Default + Clone + 'static>(
    cx: &ScopeState,
    key: impl ToString,
    init: impl FnOnce() -> T,
) -> &UsePersistent<T> {
    let mut init = Some(init);
    let state = {
        #[cfg(feature = "ssr")] 
        {
            use_ref(cx, || {
                StorageEntry::<ClientStorage, T>::new(key.to_string(), init.take().unwrap()(), false)
            })
        }
        #[cfg(all(not(feature = "ssr"), not(feature = "hydrate")))]
        {
            let state = use_signal(cx, || {
                StorageEntry::<ClientStorage, T>::new(
                    key.to_string(),
                    storage_entry::<ClientStorage, T>(key.to_string(), init.take().unwrap()),
                    Some(cx),
                )
            });
            if let StorageEntryChannel::Active(channel) = &state.read().channel {
                use_listen_channel(cx, channel, move |_| async move { state.write().update() });
            }
            state
        }
        #[cfg(all(not(feature = "ssr"), feature = "hydrate"))]
        {
            let state = use_ref(cx, || {
                StorageEntry::<ClientStorage, T>::new(key.to_string(), init.take().unwrap()(), true)
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
    cx.use_hook(|| UsePersistent {
        inner: state,
    })
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
) -> &UsePersistent<T> {
    let key = cx.use_hook(|| {
        let caller = std::panic::Location::caller();
        format!("{}:{}", caller.file(), caller.line())
    });
    log::info!("key: \"{}\"", key);
    use_persistent(cx, key, init)
}

pub struct StorageRef<'a, T: Serialize + DeserializeOwned + Default + Clone + 'static> {
    inner: Ref<'a, StorageEntry<ClientStorage, T>>,
}

impl<'a, T: Serialize + DeserializeOwned + Default + Clone + 'static> Deref for StorageRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct StorageRefMut<'a, T: Serialize + DeserializeOwned + Clone + 'static> {
    inner: Write<'a, StorageEntry<ClientStorage, T>>,
}

impl<'a, T: Serialize + DeserializeOwned + Clone + 'static> Deref for StorageRefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T: Serialize + DeserializeOwned + Clone + 'static> DerefMut for StorageRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner.data
    }
}

impl<'a, T: Serialize + DeserializeOwned + Clone + 'static> Drop for StorageRefMut<'a, T> {
    fn drop(&mut self) {
        self.inner.deref_mut().save();
    }
}

/// Storage that persists across application reloads
pub struct UsePersistent<T: Serialize + DeserializeOwned + Default + Clone + 'static> {
    inner: Signal<StorageEntry<ClientStorage, T>>,
}

impl<T: Serialize + DeserializeOwned + Default + Clone + 'static> UsePersistent<T> {
    /// Returns a reference to the value
    pub fn read(&self) -> StorageRef<T> {
        StorageRef {
            inner: self.inner.read(),
        }
    }

    /// Returns a mutable reference to the value
    pub fn write(&self) -> StorageRefMut<T> {
        StorageRefMut {
            inner: self.inner.write(),
        }
    }

    /// Sets the value
    pub fn set(&self, value: T) {
        *self.write() = value;
        self.inner.write().save();
    }

    /// Modifies the value
    pub fn modify<F: FnOnce(&mut T)>(&self, f: F) {
        f(&mut self.write());
    }
}

impl<T: Serialize + DeserializeOwned + Default + Clone + 'static> UsePersistent<T> {
    /// Returns a clone of the value
    pub fn get(&self) -> T {
        self.read().clone()
    }
}

impl<T: Serialize + DeserializeOwned + Default + Display + Clone + 'static> Display for UsePersistent<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (*self.read()).fmt(f)
    }
}

impl<T: Serialize + DeserializeOwned + Default + Clone + 'static> Deref for UsePersistent<T> {
    type Target = Signal<StorageEntry<ClientStorage, T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Serialize + DeserializeOwned + Default + Clone + 'static> DerefMut for UsePersistent<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
