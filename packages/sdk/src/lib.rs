//! # Dioxus SDK
//!  The Dioxus SDK is a group of platform agnostic crates for common apis and functionality.
//!
//! This crate, `dioxus-sdk`, acts as an entrypoint to explore the variety of crates in the SDK ecosystem.
//! Individual crates from the SDK ecosystem can be used directly from `crates.io` or you can enable the
//! corresponding feature for a crate here.
//!
//! SDK is growing, and not all functionality supports every platform. Platform support will be documented in
//! each crate, and in the majority of cases a runtime `Err(Unsupported)` will be returned if you target an unsupported platform.
//!
//! ## Available Crates
//! Below is a table of the crates in our ecosystem, a short description, and their corresponding feature flag.
//!
//! | Crate                     | Description                           | Feature           |
//! | ------------------------- | ------------------------------------- | ----------------- |
//! | [`dioxus-geolocation`]    | Access user location services.        | `geolocation`     |
//! | [`dioxus-storage`]        | Store local and persistent data.      | `storage`         |
//! | [`dioxus-time`]           | Common timing utilities.              | `time`            |
//! | [`dioxus-window`]         | Common window utilities.              | `window`          |
//! | [`dioxus-notification`]   | Send notifications.                   | `notification`    |
//! | [`dioxus-sync`]           | Synchronization primities for Dioxus. | `sync`            |
//!
//! [`dioxus-geolocation`]: https://dioxuslabs.com
//! [`dioxus-storage`]: https://dioxuslabs.com
//! [`dioxus-time`]: https://dioxuslabs.com
//! [`dioxus-window`]: https://dioxuslabs.com
//! [`dioxus-notification`]: https://dioxuslabs.com
//! [`dioxus-sync`]: https://dioxuslabs.com

#[cfg(feature = "geolocation")]
pub use dioxus_geolocation;

#[cfg(feature = "notification")]
pub use dioxus_notification;

#[cfg(feature = "storage")]
pub use dioxus_storage;

#[cfg(feature = "sync")]
pub use dioxus_sync;

#[cfg(feature = "time")]
pub use dioxus_time;

#[cfg(feature = "window")]
pub use dioxus_window;
