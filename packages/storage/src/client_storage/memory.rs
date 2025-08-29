use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::Arc;

use crate::{StorageBacking, StorageEncoder, StoragePersistence};

#[derive(Clone)]
pub struct SessionStorage;

impl<T: Clone + Any + Send + Sync> StorageBacking<T> for SessionStorage {
    type Encoder = ArcEncoder;
    type Persistence = SessionStorage;
}

type Key = String;
type Value = Option<Arc<dyn Any>>;

fn store<T>(key: &Key, value: &Value, _unencoded: &T) {
    let session = SessionStore::get_current_session();
    match value {
        Some(value) => {
            session.borrow_mut().insert(key.clone(), value.clone());
        }
        None => {
            session.borrow_mut().remove(key);
        }
    }
}

impl StoragePersistence for SessionStorage {
    type Key = Key;
    type Value = Value;

    fn load(key: &Self::Key) -> Self::Value {
        let session = SessionStore::get_current_session();
        let read_binding = session.borrow();
        read_binding.get(key).cloned()
    }

    fn store<T>(key: &Self::Key, value: &Self::Value, unencoded: &T) {
        store(key, value, unencoded);
    }
}

/// A StorageEncoder which encodes Optional data by cloning it's content into `Arc<dyn Any>`
pub struct ArcEncoder;

impl<T: Clone + Any> StorageEncoder<T> for ArcEncoder {
    type EncodedValue = Arc<dyn Any>;
    type DecodeError = ();

    fn deserialize(loaded: &Self::EncodedValue) -> Result<T, ()> {
        let v: &Arc<dyn Any> = loaded;
        // TODO: Better error message
        v.downcast_ref::<T>().cloned().ok_or(())
    }

    fn serialize(value: &T) -> Self::EncodedValue {
        Arc::new(value.clone())
    }
}

/// An in-memory session store that is tied to the current Dioxus root context.
#[derive(Clone)]
struct SessionStore {
    /// The underlying map of session data.
    map: Rc<RefCell<HashMap<String, Arc<dyn Any>>>>,
}

impl SessionStore {
    fn new() -> Self {
        Self {
            map: Rc::new(RefCell::new(HashMap::<String, Arc<dyn Any>>::new())),
        }
    }

    /// Get the current session store from the root context, or create a new one if it doesn't exist.
    fn get_current_session() -> Self {
        dioxus::prelude::consume_context_from_scope::<Self>(dioxus::prelude::ScopeId::ROOT)
            .map_or_else(
                || {
                    let session = Self::new();
                    dioxus::prelude::provide_root_context(session.clone());
                    session
                },
                |s| s,
            )
    }
}

impl Deref for SessionStore {
    type Target = Rc<RefCell<HashMap<String, Arc<dyn Any>>>>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for SessionStore {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}
