//! # Dioxus Time Utilities
//!
//! Cross-platform timing utilities for your Dioxus apps.
//!
//! We currently offer:
//! - [`use_timeout`]
//! - [`use_debounce`]
//! - [`use_interval`]
//! - and [`sleep`]
#![warn(missing_docs)]

use std::time::Duration;

mod interval;
pub use interval::{use_interval, UseInterval};

mod debounce;
pub use debounce::{use_debounce, UseDebounce};

mod timeout;
pub use timeout::{use_timeout, UseTimeout, TimeoutHandle};

/// Pause the current task for the specified duration.
///
/// # Examples
/// ```rust
/// use std::time::Duration;
/// use dioxus::prelude::*;
///
/// #[component]
/// pub fn App() -> Element {
///     let mut has_slept = use_signal(|| false);
///     
///     use_effect(move || {
///         spawn(async move {
///             dioxus_time::sleep(Duration::from_secs(3)).await;
///             has_slept.set(true);
///         });
///     });
///
///     rsx! {
///         "I have slept: {has_slept}"
///     }
/// }
/// ```
pub async fn sleep(duration: Duration) {
    #[cfg(not(target_family = "wasm"))]
    tokio::time::sleep(duration).await;

    #[cfg(target_family = "wasm")]
    gloo_timers::future::sleep(duration).await;
}
