//! Essentially the use_ref hook except Send + Sync using Arc and RwLock.
use dioxus::prelude::*;
use std::sync::{Arc, RwLock, RwLockReadGuard};

pub fn use_rw<T: Send + Sync + 'static>(init_rw: impl FnOnce() -> T) -> UseRw<T> {
    use_hook(|| UseRw {
        update: Signal::new(schedule_update()),
        value: Signal::new(Arc::new(RwLock::new(init_rw()))),
    })
}

#[derive(Copy)]
pub struct UseRw<T: 'static> {
    update: Signal<Arc<dyn Fn() + Send + Sync + 'static>>,
    value: Signal<Arc<RwLock<T>>>,
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
        let rw_lock = self.value.read();
        rw_lock.read().map_err(|_| UseRwError::Poisoned)
    }

    pub fn write(&self, new_value: T) -> Result<(), UseRwError> {
        let rw_lock = self
            .value
            .read();
        let mut lock = rw_lock
            .write()
            .map_err(|_| UseRwError::Poisoned)?;
        *lock = new_value;
        self.needs_update();
        Ok(())
    }

    pub fn needs_update(&self) {
        (self.update.read())()
    }
}

#[derive(Debug)]
pub enum UseRwError {
    Poisoned,
}
