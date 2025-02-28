//! Send desktop notifications.
//!
//! This crate only supports desktop targets (Windows, MacOS, & Linux).
#![deny(missing_docs)]

use std::{
    error::Error,
    fmt::{self, Display},
    path::{Path, PathBuf},
};

/// Provides a builder API and contains relevant notification info.
///
/// # Examples
///
/// ```
/// use dioxus_notification::Notification;
///
/// Notification::new()
///     .app_name("dioxus test".to_string())
///     .summary("hi, this is dioxus test".to_string())
///     .body("lorem ipsum??".to_string())
///     .show()
///     .unwrap();
///
/// ```
#[derive(Debug, Clone, Default)]
pub struct Notification {
    app_name: String,
    summary: String,
    body: String,
    icon_path: PathBuf,
    timeout: NotificationTimeout,
}

/// Represents the notification's timeout.
#[derive(Debug, PartialEq, Clone, Default)]
pub enum NotificationTimeout {
    /// Default depends on the target OS.
    #[default]
    Default,
    /// A notification that has to be manually acknowledged.
    Never,
    /// A notification that times out after a duration.
    Duration(std::time::Duration),
}

cfg_if::cfg_if! {
    if #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))] {
        use notify_rust::Timeout;
        impl From<NotificationTimeout> for Timeout {
            fn from(value: NotificationTimeout) -> Self {
                match value {
                    NotificationTimeout::Default => Timeout::Default,
                    NotificationTimeout::Never => Timeout::Never,
                    NotificationTimeout::Duration(dur) => Timeout::Milliseconds(dur.as_millis().try_into().unwrap()),
                }
            }
        }
    }
}

impl Notification {
    /// Creates a new notification with empty/default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the final notification.
    pub fn show(&self) -> Result<(), NotificationError> {
        self.show_inner()
    }

    // Unsupported fallback.
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn show_inner(&self) -> Result<(), NotificationError> {
        Err(NotificationError::Unsupported)
    }

    // notify_rust implementation supporting windows, mac, and linux.
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    fn show_inner(&self) -> Result<(), NotificationError> {
        let icon_path =
            self.icon_path
                .as_os_str()
                .to_str()
                .ok_or(NotificationError::FailedToShow(
                    "failed to convert icon path into str".into(),
                ))?;

        notify_rust::Notification::new()
            .appname(&self.app_name)
            .summary(&self.summary)
            .body(&self.body)
            .icon(icon_path)
            .timeout(self.timeout.clone())
            .show()
            .map_err(|e| NotificationError::FailedToShow(e.into()))?;

        Ok(())
    }

    /// Set the application's name for the notification.
    pub fn app_name(&mut self, value: impl ToString) -> &mut Self {
        self.app_name = value.to_string();
        self
    }

    /// Set the summary content of the notification.
    pub fn summary(&mut self, value: impl ToString) -> &mut Self {
        self.summary = value.to_string();
        self
    }

    /// Set the body content of the notification.
    pub fn body(&mut self, value: impl ToString) -> &mut Self {
        self.body = value.to_string();
        self
    }

    /// Set full path to image.
    ///
    /// Not supported on MacOS.
    pub fn icon_path(&mut self, value: impl AsRef<Path>) -> &mut Self {
        self.icon_path = value.as_ref().to_path_buf();
        self
    }

    /// Set a timeout for when the notification should hide.
    pub fn timeout(&mut self, value: NotificationTimeout) -> &mut Self {
        self.timeout = value;
        self
    }
}

#[test]
#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
fn test_notification() {
    Notification::new()
        .app_name("dioxus test".to_string())
        .summary("hi, this is dioxus test".to_string())
        .body("lorem ipsum??".to_string())
        .show()
        .unwrap();
}

/// Represents errors when utilizing the notification abstraction.
#[derive(Debug)]
pub enum NotificationError {
    /// Notification is unsupported on this platform.
    Unsupported,
    /// Failure to show a notification.
    FailedToShow(Box<dyn Error>),
}

impl Error for NotificationError {}
impl Display for NotificationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Unsupported => write!(f, "notification is not supported on this platform"),
            Self::FailedToShow(err) => write!(f, "failed to show notification: {err}"),
        }
    }
}
