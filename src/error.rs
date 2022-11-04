use std::fmt;

/// The error enum that will be returned on any error when utilizing dioxus-std.
#[derive(Debug)]
pub enum DioxusStdError {
    /// Represents an error related to the [`crate::library::clipboard`] abstraction.
    Clipboard(String),
    /// Represents an error related to the [`crate::library::notification`] abstraction.
    Notification(String),
}

impl std::error::Error for DioxusStdError {}

impl fmt::Display for DioxusStdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DioxusStdError::Clipboard(s) => write!(f, "clipboard error: {}", s),
            DioxusStdError::Notification(s) => write!(f, "notification error: {}", s),
        }
    }
}
