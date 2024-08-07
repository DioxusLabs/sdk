//! Interact with location services.

cfg_if::cfg_if! {
    if #[cfg(any(windows, target_family = "wasm"))] {
        pub mod core;
        pub mod platform;
        pub mod use_geolocation;
        pub use self::core::*;
        pub use self::use_geolocation::*;
    }
    else {
        compile_error!("the `geolocation` feature is only available on wasm and windows targets");
    }
}
