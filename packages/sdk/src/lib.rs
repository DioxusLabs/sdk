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
//! | [`dioxus-sdk-geolocation`]    | Access user location services.        | `geolocation`     |
//! | [`dioxus-sdk-storage`]        | Store local and persistent data.      | `storage`         |
//! | [`dioxus-sdk-time`]           | Common timing utilities.              | `time`            |
//! | [`dioxus-sdk-window`]         | Common window utilities.              | `window`          |
//! | [`dioxus-sdk-notification`]   | Send notifications.                   | `notification`    |
//! | [`dioxus-sdk-sync`]           | Synchronization primities for Dioxus. | `sync`            |
//! | [`dioxus-sdk-util`]           | Misc utilities for Dioxus.            | `util`            |
//!
//! [`dioxus-sdk-geolocation`]: https://crates.io/crates/dioxus-sdk-geolocation
//! [`dioxus-sdk-storage`]: https://crates.io/crates/dioxus-sdk-storage
//! [`dioxus-sdk-time`]: https://crates.io/crates/dioxus-sdk-time
//! [`dioxus-sdk-window`]: https://crates.io/crates/dioxus-sdk-window
//! [`dioxus-sdk-notification`]: https://crates.io/crates/dioxus-sdk-notification
//! [`dioxus-sdk-sync`]: https://crates.io/crates/dioxus-sdk-sync
//! [`dioxus-sdk-util`]: https://crates.io/crates/dioxus-sdk-util

#[cfg(feature = "geolocation")]
pub use dioxus_sdk_geolocation as geolocation;

#[cfg(feature = "notification")]
pub use dioxus_sdk_notification as notification;

#[cfg(feature = "storage")]
pub use dioxus_sdk_storage as storage;

#[cfg(feature = "sync")]
pub use dioxus_sdk_sync as sync;

#[cfg(feature = "time")]
pub use dioxus_sdk_time as time;

#[cfg(feature = "util")]
pub use dioxus_sdk_util as util;

#[cfg(feature = "window")]
pub use dioxus_sdk_window as window;
