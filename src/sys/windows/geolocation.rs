use crate::library::geolocation::{
    DeviceStatus, Geocoordinates, GeolocationAccess, GeolocationError,
};
use windows::{
    Devices::Geolocation::{
        BasicGeoposition, GeolocationAccessStatus, Geolocator as WindowsGeolocator,
        PositionChangedEventArgs, PositionStatus, StatusChangedEventArgs,
    },
    Foundation::TypedEventHandler,
};

pub fn get_geolocator(
    report_interval: u32,
    movement_threshold: u32,
) -> Result<WindowsGeolocator, GeolocationError> {
    let geolocator =
        WindowsGeolocator::new().map_err(|e| GeolocationError::DeviceError(e.to_string()))?;

    // Set report interval
    geolocator
        .SetReportInterval(report_interval)
        .map_err(|e| GeolocationError::DeviceError(e.to_string()))?;

    // Set movement threshold
    geolocator
        .SetMovementThreshold(movement_threshold as f64)
        .map_err(|e| GeolocationError::DeviceError(e.to_string()))?;

    Ok(geolocator)
}

pub fn request_access() -> Result<GeolocationAccess, GeolocationError> {
    let access_status = match WindowsGeolocator::RequestAccessAsync() {
        Ok(v) => v,
        Err(e) => return Err(GeolocationError::DeviceError(e.to_string())),
    };

    let access_status = match access_status.get() {
        Ok(v) => v,
        Err(e) => return Err(GeolocationError::DeviceError(e.to_string())),
    };

    match access_status {
        GeolocationAccessStatus::Allowed => Ok(GeolocationAccess::Allowed),
        GeolocationAccessStatus::Denied => Ok(GeolocationAccess::Denied),
        _ => Ok(GeolocationAccess::Unspecified),
    }
}

pub fn get_coordinates(geolocator: &WindowsGeolocator) -> Result<Geocoordinates, GeolocationError> {
    let location = geolocator.GetGeopositionAsync();

    let location = match location {
        Ok(v) => v,
        Err(e) => return Err(GeolocationError::FailedToFetchCoordinates(e.to_string())),
    };

    let location = match location.get() {
        Ok(v) => v,
        Err(e) => return Err(GeolocationError::FailedToFetchCoordinates(e.to_string())),
    };

    let location_coordinate = match location.Coordinate() {
        Ok(v) => v,
        Err(e) => return Err(GeolocationError::FailedToFetchCoordinates(e.to_string())),
    };

    let location_point = match location_coordinate.Point() {
        Ok(v) => v,
        Err(e) => return Err(GeolocationError::FailedToFetchCoordinates(e.to_string())),
    };

    let position = match location_point.Position() {
        Ok(v) => v,
        Err(e) => return Err(GeolocationError::FailedToFetchCoordinates(e.to_string())),
    };

    Ok(position.into())
}

pub fn subscribe_status_changed<F: Fn(DeviceStatus) + Send + Sync + 'static>(
    geolocator: &WindowsGeolocator,
    callback: F,
) -> Result<(), GeolocationError> {
    // Subcribe to event
    let result = geolocator.StatusChanged(&TypedEventHandler::new(
        move |_geolocator: &Option<WindowsGeolocator>,
              event_args: &Option<StatusChangedEventArgs>| {
            if let Some(status_event) = event_args {
                let status = status_event.Status()?;

                (callback)(status.into())
            }
            Ok(())
        },
    ));

    // Return result
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(GeolocationError::DeviceError(e.to_string())),
    }
}

pub fn subscribe_position_changed<F: Fn(Geocoordinates) + Send + Sync + 'static>(
    geolocator: &WindowsGeolocator,
    callback: F,
) -> Result<(), GeolocationError> {
    // Subscribe to event
    let result = geolocator.PositionChanged(&TypedEventHandler::new(
        move |_geolocator: &Option<WindowsGeolocator>,
              event_args: &Option<PositionChangedEventArgs>| {
            if let Some(position) = event_args {
                // Get coordinate
                let position = position.Position()?.Coordinate()?.Point()?.Position()?;

                // Run callback
                (callback)(position.into())
            }
            Ok(())
        },
    ));

    // Return result
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(GeolocationError::DeviceError(e.to_string())),
    }
}

impl From<PositionStatus> for DeviceStatus {
    fn from(value: PositionStatus) -> Self {
        match value.0 {
            0 => DeviceStatus::Ready,
            1 => DeviceStatus::Initializing,
            3 => DeviceStatus::Disabled,
            5 => DeviceStatus::NotAvailable,
            _ => DeviceStatus::Unknown,
        }
    }
}

impl From<BasicGeoposition> for Geocoordinates {
    fn from(position: BasicGeoposition) -> Self {
        Geocoordinates {
            latitude: position.Latitude,
            longitude: position.Longitude,
            altitude: position.Altitude,
        }
    }
}

/*
TODO
- Implement status changed subscriber (defines the geolocator's ability to provide location data)
- Implement position changed subscriber (provides updates on devices' location based on report interval and movement threshold)
- Implement use_geolocation hook
- Create an initialization function to easily add geolocation to dioxus app using provide context api.
 */
