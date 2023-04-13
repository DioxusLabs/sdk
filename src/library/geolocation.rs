use windows::{
    Devices::Geolocation::{
        GeolocationAccessStatus, Geolocator as WindowsGeolocator, PositionChangedEventArgs,
        StatusChangedEventArgs,
    },
    Foundation::TypedEventHandler,
};

use crate::DioxusStdError;

#[derive(Debug)]
pub enum GeolocationError {
    AccessDenied,
    AccessUnspecified,
    Unknown,
    FailedToFetchCoordinates,
}

pub struct Geocoordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}

pub struct Geolocator {
    permission_granted: bool,
    device_geolocator: WindowsGeolocator,
}

impl Geolocator {
    pub fn new() -> Result<Self, DioxusStdError> {
        Self::request_access()
    }

    pub fn request_access() -> Result<Self, DioxusStdError> {
        // if cfg!(target_os = "windows") {}

        // Request the access from Windows crate.
        let access_status = WindowsGeolocator::RequestAccessAsync();

        let access_status = match access_status {
            Ok(v) => v,
            Err(_) => return Err(DioxusStdError::Geolocation(GeolocationError::Unknown)),
        };

        let access_status = match access_status.get() {
            Ok(v) => v,
            Err(_) => return Err(DioxusStdError::Geolocation(GeolocationError::Unknown)),
        };

        // Determine access status, return if error
        match access_status {
            GeolocationAccessStatus::Unspecified => {
                return Err(DioxusStdError::Geolocation(
                    GeolocationError::AccessUnspecified,
                ))
            }
            GeolocationAccessStatus::Denied => {
                return Err(DioxusStdError::Geolocation(GeolocationError::AccessDenied))
            }
            GeolocationAccessStatus::Allowed => true,
            _ => {
                return Err(DioxusStdError::Geolocation(
                    GeolocationError::AccessUnspecified,
                ))
            }
        };

        // Get windows geolocator
        let windows_geolocator = match WindowsGeolocator::new() {
            Ok(v) => v,
            Err(_) => return Err(DioxusStdError::Geolocation(GeolocationError::Unknown)),
        };

        // Initialize Self
        let geolocator = Self {
            permission_granted: true,
            device_geolocator: windows_geolocator,
        };

        // Initiate windows event handlers
        // StatusChanged handler (handles permission changes)
        let result = geolocator
            .device_geolocator
            .StatusChanged(&TypedEventHandler::new(
                |geolocator: &Option<WindowsGeolocator>,
                 event_args: &Option<StatusChangedEventArgs>| { Ok(()) },
            ));

        if result.is_err() {
            return Err(DioxusStdError::Geolocation(GeolocationError::Unknown));
        }

        Ok(geolocator)
    }

    pub fn start_tracking(mut self, interval: u32) -> Result<(), DioxusStdError> {
        if self.device_geolocator.SetReportInterval(interval).is_err() {
            return Err(DioxusStdError::Geolocation(GeolocationError::Unknown));
        }

        let result = self
            .device_geolocator
            .PositionChanged(&TypedEventHandler::new(
                |geolocator: &Option<WindowsGeolocator>,
                 event_args: &Option<PositionChangedEventArgs>| Ok({}),
            ));

        if result.is_err() {
            return Err(DioxusStdError::Geolocation(GeolocationError::Unknown));
        }

        Ok(())
    }

    pub fn get_current_coordinates(self) -> Result<Geocoordinates, DioxusStdError> {
        let geolocation = self.device_geolocator.GetGeopositionAsync();

        let geolocation = match geolocation {
            Ok(v) => v,
            Err(_) => {
                return Err(DioxusStdError::Geolocation(
                    GeolocationError::FailedToFetchCoordinates,
                ))
            }
        };

        let geolocation = match geolocation.get() {
            Ok(v) => v,
            Err(_) => {
                return Err(DioxusStdError::Geolocation(
                    GeolocationError::FailedToFetchCoordinates,
                ))
            }
        };

        let geolocation_coordinate = match geolocation.Coordinate() {
            Ok(v) => v,
            Err(_) => {
                return Err(DioxusStdError::Geolocation(
                    GeolocationError::FailedToFetchCoordinates,
                ))
            }
        };

        let geolocation_point = match geolocation_coordinate.Point() {
            Ok(v) => v,
            Err(_) => {
                return Err(DioxusStdError::Geolocation(
                    GeolocationError::FailedToFetchCoordinates,
                ))
            }
        };

        let position = match geolocation_point.Position() {
            Ok(v) => v,
            Err(_) => {
                return Err(DioxusStdError::Geolocation(
                    GeolocationError::FailedToFetchCoordinates,
                ))
            }
        };

        Ok(Geocoordinates {
            latitude: position.Latitude,
            longitude: position.Longitude,
            altitude: position.Altitude,
        })
    }
}

#[test]
fn test_geolocator() {
    let _geolocator = Geolocator::request_access();
}
