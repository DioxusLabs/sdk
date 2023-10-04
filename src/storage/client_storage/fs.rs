use once_cell::sync::OnceCell;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::Write;

use crate::storage::storage::{
    serde_to_string, try_serde_from_string, StorageBacking,
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
#[macro_export]
macro_rules! set_dir {
    () => {
        $crate::set_dir_name(env!("CARGO_PKG_NAME"));
    };
    ($path: literal) => {
        $crate::set_dir(std::path::PathBuf::from($path));
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
    {
        set_directory(
            directories::BaseDirs::new()
                .unwrap()
                .data_local_dir()
                .join(name),
        )
    }
}

static LOCATION: OnceCell<std::path::PathBuf> = OnceCell::new();

fn set<T: Serialize>(key: String, value: &T) {
    #[cfg(not(feature = "ssr"))]
    {
        let as_str = serde_to_string(value);
        let path = LOCATION
            .get()
            .expect("Call the set_dir macro before accessing persistant data");
        std::fs::create_dir_all(path).unwrap();
        let file_path = path.join(key);
        let mut file = std::fs::File::create(file_path).unwrap();
        file.write_all(as_str.as_bytes()).unwrap();
    }
}

fn get<T: DeserializeOwned>(key: &str) -> Option<T> {
    #[cfg(not(feature = "ssr"))]
    {
        let path = LOCATION
            .get()
            .expect("Call the set_dir macro before accessing persistant data")
            .join(key);
        let s = std::fs::read_to_string(path).ok()?;
        try_serde_from_string(&s)
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
