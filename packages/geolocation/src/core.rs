//! Provides access to the target device's geolocation system.

use super::platform;
use core::fmt;
use dioxus::prelude::Coroutine;
use std::sync::Arc;

/// Describes a position in the world.
#[derive(Debug, Clone)]
pub struct Geocoordinates {
    pub latitude: f64,
    pub longitude: f64,
}

/// To conserve battery, some devices allow setting a desired accuracy based on your use-case.
#[derive(Debug)]
pub enum PowerMode {
    /// Will generally enable the on-board GPS for precise coordinates.
    High,
    /// Will generally use cell towers or WiFi beacons to determine the device's location.
    Low,
}

/// Represents a geolocation event.
#[derive(Debug)]
pub enum Event {
    /// The status of the device has changed.
    StatusChanged(Status),
    /// New coordinates are available.
    NewGeocoordinates(Geocoordinates),
}

/// Describes whether your application has access or not.
#[derive(Debug)]
pub enum Access {
    Allowed,
    Denied,
    /// This is returned when the access level was not able to be determined.
    Unspecified,
}

/// Describes the geolocation device's status.
#[derive(Debug, PartialEq)]
pub enum Status {
    /// Location service or device is ready and has geo data.
    Ready,
    /// Location service or device is disabled.
    Disabled,
    /// Location service or device is not available.
    NotAvailable,
    /// Location service or device is initializing.
    Initializing,
    /// Unable to determine location service or device status. (This shouldn't happen)
    Unknown,
}

/// Represents the geolocation abstraction.
pub struct Geolocator {
    device_geolocator: platform::Geolocator,
}

impl Geolocator {
    /// Create a new geolocator.
    pub fn new(power_mode: PowerMode) -> Result<Self, Error> {
        let mut device_geolocator = platform::Geolocator::new()?;
        platform::set_power_mode(&mut device_geolocator, power_mode)?;

        Ok(Self { device_geolocator })
    }

    /// Get the latest coordinates from the device.
    pub async fn get_coordinates(&self) -> Result<Geocoordinates, Error> {
        platform::get_coordinates(&self.device_geolocator).await
    }

    /// Subscribe a mpsc channel to the events.
    pub fn listen(&self, listener: Coroutine<Event>) -> Result<(), Error> {
        let tx = listener.tx();
        platform::listen(
            &self.device_geolocator,
            Arc::new(move |event: Event| {
                tx.unbounded_send(event).ok();
            }),
        )
    }
}

/// Describes errors that may occur when utilizing the geolocation abstraction.
#[derive(Debug, Clone)]
pub enum Error {
    NotInitialized,
    AccessDenied,
    Poisoned,
    DeviceError(String),
}

impl std::error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AccessDenied => {
                write!(f, "access denied (access may have been revoked during use)")
            }
            Error::Poisoned => write!(f, "the internal read/write lock has been poisioned"),
            Error::DeviceError(e) => write!(f, "a device error has occurred: {}", e),
        }
    }
}
