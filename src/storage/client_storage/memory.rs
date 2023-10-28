use once_cell::sync::Lazy;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::ops::{DerefMut, Deref};
use std::sync::Arc;
use std::{collections::HashMap, sync::RwLock};

use crate::storage::{serde_to_string, try_serde_from_string, StorageBacking};

#[derive(Clone)]
pub struct SessionStorage;

impl StorageBacking for SessionStorage {
    type Key = String;

    fn set<T: Serialize>(key: String, value: &T) {
        let session = SessionStore::get_current_session();
        session
            .write()
            .unwrap()
            .insert(key, serde_to_string(value));
    }

    fn get<T: DeserializeOwned>(key: &String) -> Option<T> {
        let session = SessionStore::get_current_session();
        let read_binding = session.read().unwrap();
        let string = read_binding.get(key)?;
        try_serde_from_string(string)
    }
}

#[derive(Clone)]
struct SessionStore {
    map: Arc<RwLock<HashMap<String, String>>>,
}

impl SessionStore {
    fn new() -> Self {
        Self {
            map: Arc::new(RwLock::new(HashMap::<String, String>::new())),
        }
    }
    fn get_current_session() -> Self {
        dioxus::prelude::consume_context_from_scope::<Self>(dioxus::prelude::ScopeId::ROOT).map_or_else(|| {
            let session = Self::new();
            dioxus::prelude::provide_root_context(session.clone());
            session
        }, |s| s)

    }
}

impl Deref for SessionStore {
    type Target = Arc<RwLock<HashMap<String, String>>>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for SessionStore {
    fn deref_mut(&mut self) -> &mut Self::Target {
       &mut self.map
    }
}