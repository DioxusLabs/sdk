//! Interact with location services.
//!
//! ## Platform-specific:
//!
//! **Android / iOS / Linux / Mac:** Unsupported.
pub mod core;
pub mod platform;
pub mod use_geolocation;
pub use self::core::*;
pub use self::use_geolocation::*;
