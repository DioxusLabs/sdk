//! Provides access to the target device's geolocation system.

use std::sync::Arc;

use dioxus::prelude::Coroutine;

use crate::sys;

/// Represents the geolocation abstraction.
pub struct Geolocator {
    device_geolocator: Box<dyn DeviceGeolocator>,
}

impl Geolocator {
    /// Create a new geolocator.
    pub fn new(power_mode: PowerMode) -> Result<Self, Error> {
        let mut device_geolocator = sys::geolocation::Geolocator::new()?;
        device_geolocator.set_power_mode(power_mode)?;

        Ok(Self {
            device_geolocator: Box::new(device_geolocator),
        })
    }

    /// Get the latest coordinates from the device.
    pub fn get_coordinates(&self) -> Result<Geocoordinates, Error> {
        self.device_geolocator.get_coordinates()
    }

    /// Subscribe a mpsc channel to the events.
    pub fn listen(&self, listener: Coroutine<Event>) -> Result<(), Error> {
        self.device_geolocator.listen(Arc::new(move |event: Event| {
            listener.send(event);
        }))
    }
}

/// Represents a geolocation event.
#[derive(Debug)]
pub enum Event {
    /// The status of the device has changed.
    StatusChanged(Status),
    /// New coordinates are available.
    NewGeocoordinates(Geocoordinates),
}

pub trait DeviceGeolocator {
    fn get_coordinates(&self) -> Result<Geocoordinates, Error>;
    fn listen(&self, callback: Arc<dyn Fn(Event) + Send + Sync>) -> Result<(), Error>;
    fn set_power_mode(&mut self, power_mode: PowerMode) -> Result<(), Error>;
}

/// Describes a position in the world.
#[derive(Debug, Clone)]
pub struct Geocoordinates {
    pub latitude: f64,
    pub longitude: f64,
}

/// Describes whether your application has access or not.
#[derive(Debug)]
pub enum Access {
    Allowed,
    Denied,
    /// This is returned when the access level was not able to be determined.
    Unspecified,
}

/// Describes errors that may occur when utilizing the geolocation abstraction.
#[derive(Debug, Clone)]
pub enum Error {
    NotInitialized,
    AccessDenied,
    Poisoned,
    DeviceError(String),
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

/// To conserve battery, some devices allow setting a desired accuracy based on your use-case.
#[derive(Debug)]
pub enum PowerMode {
    /// Will generally enable the on-board GPS for precise coordinates.
    High,
    /// Will generally use cell towers or WiFi beacons to determine the device's location.
    Low,
}
