use std::fmt;

/// The error enum that will be returned on any error when utilizing dioxus-std.
#[derive(Debug)]
pub enum DioxusStdError {
    /// The Clipboard enum specifies an error related to the [`crate::library::clipboard::Clipboard`] abstraction.
    Clipboard(String),
}

impl std::error::Error for DioxusStdError {}

impl fmt::Display for DioxusStdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DioxusStdError::Clipboard(s) => write!(f, "clipboard error: {}", s),
        }
    }
}
