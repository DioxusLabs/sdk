#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::*;

#[cfg(target_family = "wasm")]
mod wasm;
#[cfg(target_family = "wasm")]
pub use self::wasm::*;

#[cfg(not(any(target_family = "wasm", windows)))]
mod unsupported {
    use std::sync::Arc;

    use crate::{Error, Event, Geocoordinates};

    pub struct Geolocator {}

    impl Geolocator {
        /// Create a new Geolocator for the device.
        pub fn new() -> Result<Self, Error> {
            Err(Error::Unsupported)
        }
    }

    pub async fn get_coordinates(_geolocator: &Geolocator) -> Result<Geocoordinates, Error> {
        Err(Error::Unsupported)
    }

    pub fn set_power_mode(
        _geolocator: &mut Geolocator,
        _power_mode: crate::PowerMode,
    ) -> Result<(), Error> {
        Err(Error::Unsupported)
    }

    pub fn listen(
        _geolocator: &Geolocator,
        _callback: Arc<dyn Fn(Event) + Send + Sync>,
    ) -> Result<(), Error> {
        Err(Error::Unsupported)
    }
}

#[cfg(not(any(target_family = "wasm", windows)))]
pub use self::unsupported::*;
