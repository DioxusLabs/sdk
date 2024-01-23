//! Essentially the use_ref hook except Send + Sync using Arc and RwLock.
use dioxus::prelude::*;
use std::sync::{Arc, RwLock, RwLockReadGuard};

pub fn use_rw<T: Send + Sync + 'static>(init_rw: impl FnOnce() -> T) -> UseRw<T> {
    use_hook(|| UseRw {
        update: schedule_update(),
        value: Arc::new(RwLock::new(init_rw())),
    })
}

pub struct UseRw<T> {
    update: Arc<dyn Fn() + Send + Sync + 'static>,
    value: Arc<RwLock<T>>,
}

impl<T> Clone for UseRw<T> {
    fn clone(&self) -> Self {
        Self {
            update: self.update.clone(),
            value: self.value.clone(),
        }
    }
}

impl<T> UseRw<T> {
    pub fn read(&self) -> Result<RwLockReadGuard<'_, T>, UseRwError> {
        self.value.read().map_err(|_| UseRwError::Poisoned)
    }

    pub fn write(&self, new_value: T) -> Result<(), UseRwError> {
        let mut lock = self.value.write().map_err(|_| UseRwError::Poisoned)?;
        *lock = new_value;
        self.needs_update();
        Ok(())
    }

    pub fn needs_update(&self) {
        (self.update)()
    }
}

#[derive(Debug)]
pub enum UseRwError {
    Poisoned,
}
