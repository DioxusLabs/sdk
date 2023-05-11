use crate::library::geolocation::{Geocoordinates, GeolocationAccess, GeolocationError};
use windows::Devices::Geolocation::{GeolocationAccessStatus, Geolocator as WindowsGeolocator};

pub fn get_geolocator() -> Result<WindowsGeolocator, GeolocationError> {
    WindowsGeolocator::new().map_err(|e| GeolocationError::DeviceError(e.to_string()))
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

    Ok(Geocoordinates {
        latitude: position.Latitude,
        longitude: position.Longitude,
        altitude: position.Altitude,
    })
}
