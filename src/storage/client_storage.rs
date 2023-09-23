#![allow(unused)]
use dioxus::prelude::*;
use once_cell::sync::OnceCell;
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
#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! set_dir {
    () => {
        $crate::set_dir_name(env!("CARGO_PKG_NAME"));
    };
    ($path: literal) => {
        $crate::set_dir(std::path::PathBuf::from($path));
    };
}

#[cfg(not(target_arch = "wasm32"))]
#[doc(hidden)]
/// Sets the directory where the storage files are located.
pub fn set_directory(path: std::path::PathBuf) {
    LOCATION.set(path).unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
#[doc(hidden)]
pub fn set_dir_name(name: &str) {
    {
        set_directory(
            directories::BaseDirs::new()
                .unwrap()
                .data_local_dir()
                .join(name),
        )
    }
}

#[cfg(not(target_arch = "wasm32"))]
static LOCATION: OnceCell<std::path::PathBuf> = OnceCell::new();

#[cfg(target_arch = "wasm32")]
fn local_storage() -> Option<Storage> {
    window()?.local_storage().ok()?
}

fn set<T: Serialize>(key: String, value: &T) {
    #[cfg(not(feature = "ssr"))]
    {
        let as_str = serde_to_string(value);
        #[cfg(target_arch = "wasm32")]
        {
            local_storage().unwrap().set_item(&key, &as_str).unwrap();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = LOCATION
                .get()
                .expect("Call the set_dir macro before accessing persistant data");
            std::fs::create_dir_all(path).unwrap();
            let file_path = path.join(key);
            let mut file = std::fs::File::create(file_path).unwrap();
            file.write_all(as_str.as_bytes()).unwrap();
        }
    }
}

fn get<T: DeserializeOwned>(key: &str) -> Option<T> {
    #[cfg(not(feature = "ssr"))]
    {
        #[cfg(target_arch = "wasm32")]
        {
            let s: String = local_storage()?.get_item(key).ok()??;
            try_serde_from_string(&s)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = LOCATION
                .get()
                .expect("Call the set_dir macro before accessing persistant data")
                .join(key);
            let s = std::fs::read_to_string(path).ok()?;
            try_serde_from_string(&s)
        }
    }
    #[cfg(feature = "ssr")]
    None
}

pub struct ClientStorage;

impl StorageBacking for ClientStorage {
    type Key = String;

    fn set<T: Serialize>(key: String, value: &T) {
        set(key, value);
    }

    fn get<T: DeserializeOwned>(key: &String) -> Option<T> {
        get(key)
    }
}

/// A persistent storage hook that can be used to store data across application reloads.
///
/// Depending on the platform this uses either local storage or a file storage
#[allow(clippy::needless_return)]
pub fn use_persistent<T: Serialize + DeserializeOwned + Default + 'static>(
    cx: &ScopeState,
    key: impl ToString,
    init: impl FnOnce() -> T,
) -> &UsePersistent<T> {
    let mut init = Some(init);
    #[cfg(feature = "ssr")]
    let state = use_ref(cx, || {
        StorageEntry::<ClientStorage, T>::new(key.to_string(), init.take().unwrap()())
    });
    // if hydration is not enabled we can just set the storage
    #[cfg(all(not(feature = "ssr"), not(feature = "hydrate")))]
    let state = use_ref(cx, || {
        StorageEntry::new(
            key.to_string(),
            storage_entry::<ClientStorage, T>(key.to_string(), init.take().unwrap()),
        )
    });
    // otherwise render the initial value and then hydrate after the first render
    #[cfg(all(not(feature = "ssr"), feature = "hydrate"))]
    let state = {
        let state = use_ref(cx, || {
            StorageEntry::<ClientStorage, T>::new(key.to_string(), init.take().unwrap()())
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
    };
    cx.use_hook(|| UsePersistent {
        inner: state.clone(),
    })
}

/// A persistent storage hook that can be used to store data across application reloads.
/// The state will be the same for every call to this hook from the same line of code.
///
/// Depending on the platform this uses either local storage or a file storage
#[allow(clippy::needless_return)]
#[track_caller]
pub fn use_singleton_persistent<T: Serialize + DeserializeOwned + Default + 'static>(
    cx: &ScopeState,
    init: impl FnOnce() -> T,
) -> &UsePersistent<T> {
    let key = cx.use_hook(|| {
        let caller = std::panic::Location::caller();
        format!("{}:{}", caller.file(), caller.line())
    });
    use_persistent(cx, key, init)
}

pub struct StorageRef<'a, T: Serialize + DeserializeOwned + Default + 'static> {
    inner: Ref<'a, StorageEntry<ClientStorage, T>>,
}

impl<'a, T: Serialize + DeserializeOwned + Default + 'static> Deref for StorageRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct StorageRefMut<'a, T: Serialize + DeserializeOwned + 'static> {
    inner: RefMut<'a, StorageEntry<ClientStorage, T>>,
}

impl<'a, T: Serialize + DeserializeOwned + 'static> Deref for StorageRefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T: Serialize + DeserializeOwned + 'static> DerefMut for StorageRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner.data
    }
}

impl<'a, T: Serialize + DeserializeOwned + 'static> Drop for StorageRefMut<'a, T> {
    fn drop(&mut self) {
        self.inner.deref_mut().save();
    }
}

/// Storage that persists across application reloads
pub struct UsePersistent<T: Serialize + DeserializeOwned + Default + 'static> {
    inner: UseRef<StorageEntry<ClientStorage, T>>,
}

impl<T: Serialize + DeserializeOwned + Default + 'static> UsePersistent<T> {
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

impl<T: Serialize + DeserializeOwned + Default + Display + 'static> Display for UsePersistent<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (*self.read()).fmt(f)
    }
}

impl<T: Serialize + DeserializeOwned + Default + 'static> Deref for UsePersistent<T> {
    type Target = UseRef<StorageEntry<ClientStorage, T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Serialize + DeserializeOwned + Default + 'static> DerefMut for UsePersistent<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
