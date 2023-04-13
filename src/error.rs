use std::fmt;

/// Represents any error when utilizing dioxus-std.
#[derive(Debug)]
pub enum DioxusStdError {
    /// Represents an error related to the [`crate::library::clipboard`] abstraction.
    Clipboard(String),
    /// Represents an error related to the [`crate::library::notification`] abstraction.
    Notification(String),
    /// Represents an error related to the [`crate::library::camera`] abstraction.
    Camera(String),

    #[cfg(feature = "geolocation")]
    /// Represents an error related to the [`crate::library::geolocation`] abstraction.
    Geolocation(crate::geolocation::GeolocationError),
}

impl std::error::Error for DioxusStdError {}

impl fmt::Display for DioxusStdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DioxusStdError::Clipboard(s) => write!(f, "clipboard error: {}", s),
            DioxusStdError::Notification(s) => write!(f, "notification error: {}", s),
            DioxusStdError::Camera(s) => write!(f, "camera error: {}", s),
            DioxusStdError::Geolocation(s) => write!(f, "geolocation error: {:?}", s),
        }
    }
}
