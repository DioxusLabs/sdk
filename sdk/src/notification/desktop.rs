//! Provides a notification abstraction to access the target system's notification feature.

use notify_rust::Timeout;
use std::fmt;

/// Provides a builder API and contains relevant notification info.
///
/// # Examples
///
/// ```
/// use dioxus_sdk::notification::Notification;
///
/// Notification::new()
///     .app_name("dioxus test".to_string())
///     .summary("hi, this is dioxus test".to_string())
///     .body("lorem ipsum??".to_string())
///     .show()
///     .unwrap();
///
/// ```
#[derive(Debug)]
pub struct Notification {
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub icon_path: String,
    pub timeout: NotificationTimeout,
}

/// Represents the notification's timeout.
#[derive(Debug, PartialEq, Clone)]
pub enum NotificationTimeout {
    /// Default depends on the target OS.
    Default,
    Never,
    Milliseconds(u32),
}

impl From<NotificationTimeout> for Timeout {
    fn from(value: NotificationTimeout) -> Self {
        match value {
            NotificationTimeout::Default => Timeout::Default,
            NotificationTimeout::Never => Timeout::Never,
            NotificationTimeout::Milliseconds(ms) => Timeout::Milliseconds(ms),
        }
    }
}

impl Notification {
    /// Creates a new notification with empty/default values.
    pub fn new() -> Self {
        Notification {
            app_name: "".to_string(),
            summary: "".to_string(),
            body: "".to_string(),
            icon_path: "".to_string(),
            timeout: NotificationTimeout::Default,
        }
    }

    /// Show the final notification.
    pub fn show(&self) -> Result<(), NotificationError> {
        let result = notify_rust::Notification::new()
            .appname(&self.app_name)
            .summary(&self.summary)
            .body(&self.body)
            .icon(&self.icon_path)
            .timeout(self.timeout.clone())
            .show();

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(NotificationError::FailedToShowNotification(e.to_string())),
        }
    }

    // Setters
    /// Set the application's name for the notification.
    pub fn app_name(&mut self, value: String) -> &mut Self {
        self.app_name = value;
        self
    }

    /// Set the summary content of the notification.
    pub fn summary(&mut self, value: String) -> &mut Self {
        self.summary = value;
        self
    }

    /// Set the body content of the notification.
    pub fn body(&mut self, value: String) -> &mut Self {
        self.body = value;
        self
    }

    /// Set full path to image.
    /// Only works on Linux.
    pub fn icon_path(&mut self, value: String) -> &mut Self {
        self.icon_path = value;
        self
    }

    /// Set a timeout for when the notification should hide.
    pub fn timeout(&mut self, value: NotificationTimeout) -> &mut Self {
        self.timeout = value;
        self
    }
}

impl Default for Notification {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
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
    /// Failure to show a notification.
    FailedToShowNotification(String),
}

impl std::error::Error for NotificationError {}
impl fmt::Display for NotificationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NotificationError::FailedToShowNotification(s) => write!(f, "{}", s),
        }
    }
}
