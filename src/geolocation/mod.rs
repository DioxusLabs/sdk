mod core;
mod platform;
mod use_geolocation;

pub mod geolocation {
    pub use super::core::*;
    pub use super::platform::*;
    pub use super::use_geolocation::*;
}
