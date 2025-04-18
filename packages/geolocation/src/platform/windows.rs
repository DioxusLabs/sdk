use std::sync::Arc;

use windows::{
    Devices::Geolocation::{
        BasicGeoposition, GeolocationAccessStatus, Geolocator as WindowsGeolocator,
        PositionAccuracy, PositionChangedEventArgs, PositionStatus, StatusChangedEventArgs,
    },
    Foundation::TypedEventHandler,
};

use crate::core::{Error, Event, Geocoordinates, PowerMode, Status};

/// Represents the HAL's geolocator.
pub struct Geolocator {
    device_geolocator: WindowsGeolocator,
}

impl Geolocator {
    /// Create a new Geolocator for the device.
    pub fn new() -> Result<Self, Error> {
        // Check access
        let access_status = match WindowsGeolocator::RequestAccessAsync() {
            Ok(v) => v,
            Err(e) => return Err(Error::DeviceError(e.to_string())),
        };

        let access_status = match access_status.get() {
            Ok(v) => v,
            Err(e) => return Err(Error::DeviceError(e.to_string())),
        };

        if access_status != GeolocationAccessStatus::Allowed {
            return Err(Error::AccessDenied);
        }

        // Get geolocator
        let device_geolocator =
            WindowsGeolocator::new().map_err(|e| Error::DeviceError(e.to_string()))?;

        Ok(Self { device_geolocator })
    }
}

pub async fn get_coordinates(geolocator: &Geolocator) -> Result<Geocoordinates, Error> {
    let location = geolocator.device_geolocator.GetGeopositionAsync();

    let location = match location {
        Ok(v) => v,
        Err(e) => return Err(Error::DeviceError(e.to_string())),
    };

    let location = match location.get() {
        Ok(v) => v,
        Err(e) => return Err(Error::DeviceError(e.to_string())),
    };

    let location_coordinate = match location.Coordinate() {
        Ok(v) => v,
        Err(e) => return Err(Error::DeviceError(e.to_string())),
    };

    let location_point = match location_coordinate.Point() {
        Ok(v) => v,
        Err(e) => return Err(Error::DeviceError(e.to_string())),
    };

    let position = match location_point.Position() {
        Ok(v) => v,
        Err(e) => return Err(Error::DeviceError(e.to_string())),
    };

    Ok(position.into())
}

/// Listen to new events with a callback.
pub fn listen(
    geolocator: &Geolocator,
    callback: Arc<dyn Fn(Event) + Send + Sync>,
) -> Result<(), Error> {
    let callback1 = callback.clone();
    let callback2 = callback.clone();

    // Subscribe to status changed
    geolocator
        .device_geolocator
        .StatusChanged(&TypedEventHandler::new(
            move |_geolocator: &Option<WindowsGeolocator>,
                  event_args: &Option<StatusChangedEventArgs>| {
                if let Some(status) = event_args {
                    // Get status
                    let status = status.Status()?;

                    // Run callback
                    (callback1)(Event::StatusChanged(status.into()))
                }
                Ok(())
            },
        ))
        .map_err(|e| Error::DeviceError(e.to_string()))?;

    // Subscribe to position changed
    geolocator
        .device_geolocator
        .PositionChanged(&TypedEventHandler::new(
            move |_geolocator: &Option<WindowsGeolocator>,
                  event_args: &Option<PositionChangedEventArgs>| {
                if let Some(position) = event_args {
                    // Get coordinate
                    let position = position.Position()?.Coordinate()?.Point()?.Position()?;

                    // Run callback
                    (callback2)(Event::NewGeocoordinates(position.into()))
                }
                Ok(())
            },
        ))
        .map_err(|e| Error::DeviceError(e.to_string()))?;

    Ok(())
}

/// Set the device's power mode.
pub fn set_power_mode(geolocator: &mut Geolocator, power_mode: PowerMode) -> Result<(), Error> {
    match power_mode {
        PowerMode::High => geolocator
            .device_geolocator
            .SetDesiredAccuracy(PositionAccuracy::High)
            .map_err(|e| Error::DeviceError(e.to_string()))?,
        PowerMode::Low => geolocator
            .device_geolocator
            .SetDesiredAccuracy(PositionAccuracy::Default)
            .map_err(|e| Error::DeviceError(e.to_string()))?,
    };

    Ok(())
}

impl From<PositionStatus> for Status {
    fn from(value: PositionStatus) -> Self {
        match value.0 {
            0 => Status::Ready,
            1 => Status::Initializing,
            3 => Status::Disabled,
            5 => Status::NotAvailable,
            _ => Status::Unknown,
        }
    }
}

impl From<BasicGeoposition> for Geocoordinates {
    fn from(position: BasicGeoposition) -> Self {
        Geocoordinates {
            latitude: position.Latitude,
            longitude: position.Longitude,
        }
    }
}
