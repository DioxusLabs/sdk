use dioxus::prelude::{use_hook, Callback, Writable};
use std::time::Duration;

/// The interface to a debounce.
///
/// This handle allows you to cancel an interval.
#[derive(Clone, PartialEq, Copy)]
pub struct UseInterval {
    inner: dioxus::prelude::Signal<InnerUseInterval>,
}

struct InnerUseInterval {
    pub(crate) interval: Option<dioxus::prelude::Task>,
}

impl Drop for InnerUseInterval {
    fn drop(&mut self) {
        if let Some(interval) = self.interval.take() {
            interval.cancel();
        }
    }
}

impl UseInterval {
    /// Cancel the interval.
    pub fn cancel(&mut self) {
        if let Some(interval) = self.inner.write().interval.take() {
            interval.cancel();
        }
    }
}

/// Repeatedly call a function at a specific interval.
///
/// Intervals are cancelable with the [`UseInterval::cancel`] method.
///
/// # Examples
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_time::use_interval;
/// use std::time::Duration;
///
/// #[component]
/// fn App() -> Element {
///     let mut time_elapsed = use_signal(|| 0);
///      use_interval(Duration::from_secs(1), move || *time_elapsed.write() += 1);
///     
///     rsx! {
///         "It has been {time_elapsed} since the app started."
///     }
/// }
/// ```
pub fn use_interval(period: Duration, mut action: impl FnMut() + 'static) -> UseInterval {
    let inner = use_hook(|| {
        let callback = Callback::new(move |()| {
            action();
        });

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
