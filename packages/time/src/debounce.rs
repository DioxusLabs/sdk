use dioxus::prelude::*;
use futures::{
    channel::mpsc::{self, UnboundedSender as Sender},
    StreamExt,
};
use std::time::Duration;

/// The interface for calling a debounce.
///
/// See [`use_debounce`] for more information.
pub struct UseDebounce<T: 'static> {
    sender: Signal<Sender<T>>,
}

impl<T> UseDebounce<T> {
    /// Start the debounce countdown, resetting it if already started.
    pub fn action(&mut self, data: T) {
        self.sender.write().unbounded_send(data).ok();
    }
}

// Manually implement Clone, Copy, and PartialEq as #[derive] thinks that T needs to implement these (it doesn't).
impl<T> Clone for UseDebounce<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for UseDebounce<T> {}

impl<T> PartialEq for UseDebounce<T> {
    fn eq(&self, other: &Self) -> bool {
        self.sender == other.sender
    }
}

/// A hook for allowing a function to be called only after a provided [`Duration`] has passed.
///
/// Once the [`UseDebounce::action`] method is called, a timer will start counting down until
/// the callback is ran. If the [`UseDebounce::action`] method is called again, the timer will restart.
///
/// # Examples
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_time::use_debounce;
/// use std::time::Duration;
///
/// #[component]
/// fn App() -> Element {
///     let mut debounce = use_debounce(Duration::from_secs(2), |_| println!("ran"));
///     
///     rsx! {
///         button {
///             onclick: move |_| {
///                 debounce.action(());
///             },
///             "Click!"
///         }
///     }
/// }
/// ```
pub fn use_debounce<T>(time: Duration, cb: impl FnOnce(T) + Copy + 'static) -> UseDebounce<T> {
    use_hook(|| {
        let (sender, mut receiver) = mpsc::unbounded();
        let debouncer = UseDebounce {
            sender: Signal::new(sender),
        };

        spawn(async move {
            let mut current_task: Option<Task> = None;

            loop {
                if let Some(data) = receiver.next().await {
                    if let Some(task) = current_task.take() {
                        task.cancel();
                    }

                    current_task = Some(spawn(async move {
                        #[cfg(not(target_family = "wasm"))]
                        tokio::time::sleep(time).await;

                        #[cfg(target_family = "wasm")]
                        gloo_timers::future::sleep(time).await;

                        cb(data);
                    }));
                }
            }
        });

        debouncer
    })
}
