use std::time::Duration;

use dioxus::prelude::{spawn, use_hook, Task, Writable};

#[derive(Clone, PartialEq, Copy)]
pub struct UseInterval {
    inner: dioxus::prelude::Signal<InnerUseInterval>,
}

struct InnerUseInterval {
    #[cfg(target_family = "wasm")]
    pub(crate) interval: Option<gloo_timers::callback::Interval>,

    #[cfg(not(target_family = "wasm"))]
    pub(crate) interval: Option<Task>,
}

#[cfg(target_family = "wasm")]
impl Drop for InnerUseInterval {
    fn drop(&mut self) {
        if let Some(interval) = self.interval.take() {
            interval.cancel();
        }
    }
}

impl UseInterval {
    /// Cancel the interval
    pub fn cancel(&mut self) {
        if let Some(interval) = self.inner.write().interval.take() {
            interval.cancel();
        }
    }
}

/// Repeatedly calls a function every a certain period.
pub fn use_interval(period: Duration, action: impl FnMut() + 'static) -> UseInterval {
    let inner = use_hook(|| {
        let mut action = Box::new(action);

        #[cfg(target_family = "wasm")]
        return dioxus::prelude::Signal::new(InnerUseInterval {
            interval: Some(gloo_timers::callback::Interval::new(
                period.as_millis() as u32,
                move || {
                    action();
                },
            )),
        });

        #[cfg(not(target_family = "wasm"))]
        dioxus::prelude::Signal::new(InnerUseInterval {
            interval: Some(spawn(async move {
                let mut interval = tokio::time::interval(period);
                loop {
                    interval.tick().await;
                    action();
                }
            })),
        })
    });

    UseInterval { inner }
}
