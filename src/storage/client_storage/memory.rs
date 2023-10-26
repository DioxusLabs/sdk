use once_cell::sync::Lazy;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use std::{collections::HashMap, sync::RwLock};

use crate::storage::{serde_to_string, try_serde_from_string, StorageBacking};

#[derive(Clone)]
pub struct SessionStorage;

impl StorageBacking for SessionStorage {
    type Key = String;

    fn set<T: Serialize>(key: String, value: &T) {
        SESSION_STORE
            .write()
            .unwrap()
            .insert(key, serde_to_string(value));
    }

    fn get<T: DeserializeOwned>(key: &String) -> Option<T> {
        let read_binding = SESSION_STORE.read().unwrap();
        let string = read_binding.get(key)?;
        try_serde_from_string(string)
    }
}

static SESSION_STORE: Lazy<Arc<RwLock<HashMap<String, String>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));
