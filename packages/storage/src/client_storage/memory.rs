use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::Arc;

use dioxus::core::{consume_context_from_scope, provide_root_context};

use crate::StorageBacking;

#[derive(Clone)]
pub struct SessionStorage;

impl StorageBacking for SessionStorage {
    type Key = String;

    fn set<T: Clone + 'static>(key: String, value: &T) {
        let session = SessionStore::get_current_session();
        session.borrow_mut().insert(key, Arc::new(value.clone()));
    }

    fn get<T: Clone + 'static>(key: &String) -> Option<T> {
        let session = SessionStore::get_current_session();
        let read_binding = session.borrow();
        let value_any = read_binding.get(key)?;
        value_any.downcast_ref::<T>().cloned()
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
        consume_context_from_scope::<Self>(dioxus::prelude::ScopeId::ROOT).map_or_else(
            || {
                let session = Self::new();
                provide_root_context(session.clone());
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
