//! Useful cross-platform abstractions for common use-cases.

#[cfg(any(feature = "clipboard", doc))]
pub mod clipboard;

#[cfg(any(feature = "notifications", doc))]
pub mod notification;

#[cfg(any(feature = "geolocation", doc))]
pub mod geolocation;
