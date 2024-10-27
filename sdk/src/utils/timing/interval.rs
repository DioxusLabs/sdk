use dioxus::prelude::{use_hook, warnings::signal_write_in_component_body, Callback, Writable};
use std::time::Duration;
use warnings::Warning;

#[derive(Clone, PartialEq, Copy)]
pub struct UseInterval {
    inner: dioxus::prelude::Signal<InnerUseInterval>,
}

struct InnerUseInterval {
    pub(crate) interval: Option<dioxus::prelude::Task>,
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
pub fn use_interval(period: Duration, mut action: impl FnMut() + 'static) -> UseInterval {
    let inner = use_hook(|| {
        let callback = Callback::new(move |()| {
            action();
        });

        // #[cfg(target_family = "wasm")]
        // return dioxus::prelude::Signal::new(InnerUseInterval {
        //     interval: Some(gloo_timers::callback::Interval::new(
        //         period.as_millis() as u32,
        //         move || {
        //             callback.call(());
        //             //action();
        //         },
        //     )),
        // });

        dioxus::prelude::Signal::new(InnerUseInterval {
            interval: Some(dioxus::prelude::spawn(async move {
                #[cfg(not(target_family = "wasm"))]
                let mut interval = tokio::time::interval(period);

                loop {
                    #[cfg(not(target_family = "wasm"))]
                    interval.tick().await;

                    #[cfg(target_family = "wasm")]
                    gloo_timers::future::sleep(period).await;

                    callback.call(());
                }
            })),
        })
    });

    UseInterval { inner }
}
