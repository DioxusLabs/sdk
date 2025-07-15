use std::sync::Arc;

use crate::core::{Error, Event, Geocoordinates, PowerMode};

/// Represents the HAL's geolocator.
pub struct Geolocator;

impl Geolocator {
    /// Create a new Geolocator for the device.
    pub fn new() -> Result<Self, Error> {
        Err(Error::Unsupported)
    }
}

pub async fn get_coordinates(_geolocator: &Geolocator) -> Result<Geocoordinates, Error> {
    Err(Error::Unsupported)
}

/// Listen to new events with a callback.
pub fn listen(
    _geolocator: &Geolocator,
    _callback: Arc<dyn Fn(Event) + Send + Sync>,
) -> Result<(), Error> {
    Err(Error::Unsupported)
}

/// Set the device's power mode.
pub fn set_power_mode(_geolocator: &mut Geolocator, _power_mode: PowerMode) -> Result<(), Error> {
    Err(Error::Unsupported)
}
