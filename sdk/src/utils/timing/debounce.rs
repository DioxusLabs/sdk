use dioxus::prelude::*;
use futures::{
    channel::mpsc::{self, UnboundedSender as Sender},
    StreamExt,
};
use std::time::{Duration, Instant};

type DebounceCallback = Box<dyn FnOnce()>;

/// The interface for calling a debounce.
///
/// See [`use_debounce`] for more information.
#[derive(Clone, Copy, PartialEq)]
pub struct UseDebounce {
    sender: Signal<Sender<DebounceCallback>>,
}

impl UseDebounce {
    /// Will run the provided function if the debounce period has passed.
    pub fn action(&mut self, cb: impl FnOnce() + 'static) {
        self.sender.write().unbounded_send(Box::new(cb)).ok();
    }
}

/// A hook for allowing a function to be called only after a provided [`Duration`] has passed.
///
/// This hook only checks if the callback can be ran when the [`UseDebounce::action`] method is called.
/// It will not queue function calls.
///
/// # Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_sdk::utils::timing::use_debounce;
/// use std::time::Duration;
///
/// fn App() -> Element {
///     let mut debounce = use_debounce(Duration::from_millis(2000));
///     
///     rsx! {
///         button {
///             onclick: move |_| {
///                 debounce.action(|| println!("ran"));
///             },
///             "Click!"
///         }
///     }
/// }
/// ```
pub fn use_debounce(wait_period: Duration) -> UseDebounce {
    use_hook(|| {
        let (sender, mut receiver) = mpsc::unbounded::<DebounceCallback>();

        spawn(async move {
            let mut last_called = None;

            loop {
                if let Some(cb) = receiver.next().await {
                    let now = Instant::now();

                    // Check if enough time has passed to run the callback.
                    if let Some(last) = last_called {
                        if now.duration_since(last) >= wait_period {
                            last_called = Some(now);
                            cb();
                        }
                    } else {
                        // Callback hasn't been ran yet.
                        last_called = Some(now);
                        cb();
                    }
                }
            }
        });

        UseDebounce {
            sender: Signal::new(sender),
        }
    })
}
