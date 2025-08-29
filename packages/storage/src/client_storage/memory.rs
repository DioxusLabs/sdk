use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::Arc;

use crate::{StorageBacking, StorageEncoder, StoragePersistence};

#[derive(Clone)]
pub struct SessionStorage;

impl<T: Clone + 'static> StorageBacking<T> for SessionStorage {
    type Key = <Self as StoragePersistence>::Key;

    fn set(key: &String, value: &T) {
        let encoded: Arc<dyn Any> = Arc::new(value.clone());
        store::<T>(key, &Some(encoded), &value);
    }

    fn get(key: &String) -> Option<T> {
        let value_any = SessionStorage::load(key)?;
        value_any.downcast_ref::<T>().cloned()
    }
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

    fn store<T: 'static + Clone>(key: &Self::Key, value: &Self::Value, unencoded: &T) {
        store(key, value, unencoded);
    }
}

/// A StorageEncoder which encodes Optional data by cloning it's content into `Arc<dyn Any>`
pub struct ArcEncoder;

impl<T: Clone + Any> StorageEncoder<Option<T>> for ArcEncoder {
    type EncodedValue = Value;

    fn deserialize(loaded: &Self::EncodedValue) -> Option<T> {
        match loaded {
            Some(v) => {
                let v: &Arc<dyn Any> = v;
                // TODO: this downcast failing is currently handled the same as `loaded` being None.
                v.downcast_ref::<T>().cloned()
            }
            None => None,
        }
    }

    fn serialize(value: &Option<T>) -> Self::EncodedValue {
        value.clone().map(|x| {
            let arc: Arc<T> = Arc::new(x);
            let arc: Arc<dyn Any> = arc;
            arc
        })
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
