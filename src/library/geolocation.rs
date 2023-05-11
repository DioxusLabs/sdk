use crate::sys;

#[cfg(windows)]
use windows::Devices::Geolocation::Geolocator as WindowsGeolocator;

/// Describes errors that may occur when utilizing the geolocation abstraction.
#[derive(Debug, Clone)]
pub enum GeolocationError {
    AccessDenied,
    DeviceError(String),
    FailedToFetchCoordinates(String),
}

/// Defines whether your application has access or not.
#[derive(Debug, PartialEq)]
pub enum GeolocationAccess {
    Allowed,
    Denied,
    Unspecified,
}

pub struct Geocoordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}

#[cfg(windows)]
type DeviceGeolocator = WindowsGeolocator;

pub struct Geolocator {
    access: GeolocationAccess,
    device_geolocator: DeviceGeolocator,
}

impl Geolocator {
    /// Create a new geolocator. This function will initialize a geolocator for the target platform and will request location permissions.
    pub fn new() -> Result<Self, GeolocationError> {
        let access = sys::geolocation::request_access()?;
        let device_geolocator = sys::geolocation::get_geolocator()?;
        Ok(Self {
            access,
            device_geolocator,
        })
    }

    pub fn request_access(&self) -> Result<GeolocationAccess, GeolocationError> {
        // Prevent double-asking
        if self.access == GeolocationAccess::Allowed {
            return Ok(GeolocationAccess::Allowed);
        }

        sys::geolocation::request_access()
    }

    pub fn get_coordinates(&self) -> Result<Geocoordinates, GeolocationError> {
        if self.access != GeolocationAccess::Allowed {
            return Err(GeolocationError::AccessDenied);
        }

        sys::geolocation::get_coordinates(&self.device_geolocator)
    }
}

/*pub struct Geolocator {
    permission_granted: bool,
    device_geolocator: WindowsGeolocator,
}

impl Geolocator {
    pub fn new() -> Result<Self, GeolocationError> {
        Self::request_access()
    }

    pub fn request_access() -> Result<Self, GeolocationError> {
        // if cfg!(target_os = "windows") {}

        // Request the access from Windows crate.
        let access_status = WindowsGeolocator::RequestAccessAsync();

        let access_status = match access_status {
            Ok(v) => v,
            Err(_) => return Err(GeolocationError::Unknown),
        };

        let access_status = match access_status.get() {
            Ok(v) => v,
            Err(_) => return Err(GeolocationError::Unknown),
        };

        // Determine access status, return if error
        match access_status {
            GeolocationAccessStatus::Unspecified => {
                return Err(GeolocationError::AccessUnspecified)
            }
            GeolocationAccessStatus::Denied => return Err(GeolocationError::AccessDenied),
            GeolocationAccessStatus::Allowed => true,
            _ => return Err(GeolocationError::AccessUnspecified),
        };

        // Get windows geolocator
        let windows_geolocator = match WindowsGeolocator::new() {
            Ok(v) => v,
            Err(_) => return Err(GeolocationError::Unknown),
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
            return Err(GeolocationError::Unknown);
        }

        Ok(geolocator)
    }

    pub fn start_tracking(mut self, interval: u32) -> Result<(), GeolocationError> {
        if self.device_geolocator.SetReportInterval(interval).is_err() {
            return Err(GeolocationError::Unknown);
        }

        let result = self
            .device_geolocator
            .PositionChanged(&TypedEventHandler::new(
                |geolocator: &Option<WindowsGeolocator>,
                 event_args: &Option<PositionChangedEventArgs>| Ok({}),
            ));

        if result.is_err() {
            return Err(GeolocationError::Unknown);
        }

        Ok(())
    }

    pub fn get_current_coordinates(self) -> Result<Geocoordinates, GeolocationError> {
        let geolocation = self.device_geolocator.GetGeopositionAsync();

        let geolocation = match geolocation {
            Ok(v) => v,
            Err(_) => return Err(GeolocationError::FailedToFetchCoordinates),
        };

        let geolocation = match geolocation.get() {
            Ok(v) => v,
            Err(_) => return Err(GeolocationError::FailedToFetchCoordinates),
        };

        let geolocation_coordinate = match geolocation.Coordinate() {
            Ok(v) => v,
            Err(_) => return Err(GeolocationError::FailedToFetchCoordinates),
        };

        let geolocation_point = match geolocation_coordinate.Point() {
            Ok(v) => v,
            Err(_) => return Err(GeolocationError::FailedToFetchCoordinates),
        };

        let position = match geolocation_point.Position() {
            Ok(v) => v,
            Err(_) => return Err(GeolocationError::FailedToFetchCoordinates),
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
*/
