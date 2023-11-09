#[cfg(any(windows, target_family = "wasm"))]
mod core;
#[cfg(any(windows, target_family = "wasm"))]
mod platform;
#[cfg(any(windows, target_family = "wasm"))]
mod use_geolocation;

#[cfg(any(windows, target_family = "wasm"))]
pub use self::core::*;
#[cfg(any(windows, target_family = "wasm"))]
pub use self::use_geolocation::*;

#[cfg(not(any(windows, target_family = "wasm")))]
compile_error!("The geolocation module is not supported on this platform.");
