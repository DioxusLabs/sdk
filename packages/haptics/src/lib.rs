//! Trigger native haptics feedback on mobile devices.

use serde::{Deserialize, Serialize};

use std::{
    error::Error,
    fmt::{self, Display},
};

/// Represents impact feedback intensity.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactFeedbackStyle {
    /// A light impact.
    Light,
    /// A medium impact.
    #[default]
    Medium,
    /// A heavy impact.
    Heavy,
    /// A soft impact.
    Soft,
    /// A rigid impact.
    Rigid,
}

impl fmt::Display for ImpactFeedbackStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ImpactFeedbackStyle::Light => "light",
            ImpactFeedbackStyle::Medium => "medium",
            ImpactFeedbackStyle::Heavy => "heavy",
            ImpactFeedbackStyle::Soft => "soft",
            ImpactFeedbackStyle::Rigid => "rigid",
        };
        write!(f, "{s}")
    }
}

/// Represents notification feedback kind.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationFeedbackType {
    /// A success notification.
    #[default]
    Success,
    /// A warning notification.
    Warning,
    /// An error notification.
    Error,
}

impl fmt::Display for NotificationFeedbackType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Error => "error",
        };
        f.write_str(s)
    }
}

/// Represents errors when utilizing the haptics abstraction.
#[derive(Debug)]
pub enum HapticsError {
    /// Haptics are unsupported on this platform.
    Unsupported,
    /// Failure to trigger the vibration.
    FailedToVibrate(Box<dyn Error>),
}

impl Error for HapticsError {}
impl Display for HapticsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Unsupported => write!(f, "haptics are not supported on this platform"),
            Self::FailedToVibrate(err) => write!(f, "failed to vibrate: {err}"),
        }
    }
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn failed_to_vibrate(err: impl Display) -> HapticsError {
    HapticsError::FailedToVibrate(std::io::Error::other(err.to_string()).into())
}

cfg_if::cfg_if! {
    if #[cfg(target_os = "android")] {
        mod android;

        /// Trigger a simple vibration.
        pub fn vibrate(duration: u32) -> Result<(), HapticsError> {
            android::vibrate(duration).map_err(failed_to_vibrate)
        }

        /// Trigger impact feedback.
        pub fn impact_feedback(style: ImpactFeedbackStyle) -> Result<(), HapticsError> {
            android::impact_feedback(style).map_err(failed_to_vibrate)
        }

        /// Trigger notification feedback.
        pub fn notification_feedback(style: NotificationFeedbackType) -> Result<(), HapticsError> {
            android::notification_feedback(style).map_err(failed_to_vibrate)
        }

        /// Trigger selection feedback.
        pub fn selection_feedback() -> Result<(), HapticsError> {
            android::selection_feedback().map_err(failed_to_vibrate)
        }
    } else if #[cfg(target_os = "ios")] {
        mod ios;

        /// Trigger a simple vibration.
        pub fn vibrate(duration: u32) -> Result<(), HapticsError> {
            ios::vibrate(duration).map_err(failed_to_vibrate)
        }

        /// Trigger impact feedback.
        pub fn impact_feedback(style: ImpactFeedbackStyle) -> Result<(), HapticsError> {
            ios::impact_feedback(style).map_err(failed_to_vibrate)
        }

        /// Trigger notification feedback.
        pub fn notification_feedback(style: NotificationFeedbackType) -> Result<(), HapticsError> {
            ios::notification_feedback(style).map_err(failed_to_vibrate)
        }

        /// Trigger selection feedback.
        pub fn selection_feedback() -> Result<(), HapticsError> {
            ios::selection_feedback().map_err(failed_to_vibrate)
        }
    } else {
        /// Trigger a simple vibration.
        pub fn vibrate(_duration: u32) -> Result<(), HapticsError> {
            Err(HapticsError::Unsupported)
        }

        /// Trigger impact feedback.
        pub fn impact_feedback(_style: ImpactFeedbackStyle) -> Result<(), HapticsError> {
            Err(HapticsError::Unsupported)
        }

        /// Trigger notification feedback.
        pub fn notification_feedback(_style: NotificationFeedbackType) -> Result<(), HapticsError> {
            Err(HapticsError::Unsupported)
        }

        /// Trigger selection feedback.
        pub fn selection_feedback() -> Result<(), HapticsError> {
            Err(HapticsError::Unsupported)
        }
    }
}
