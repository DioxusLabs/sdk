use serde::{Deserialize, Serialize};

use std::{
    error::Error,
    fmt::{self, Display},
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactFeedbackStyle {
    Light,
    #[default]
    Medium,
    Heavy,
    Soft,
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationFeedbackType {
    #[default]
    Success,
    Warning,
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
    /// Failure to show a notification.
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

cfg_if::cfg_if! {
    if #[cfg(target_os = "android")] {
        mod android;

        fn failed_to_vibrate(err: String) -> HapticsError {
            HapticsError::FailedToVibrate(std::io::Error::other(err).into())
        }

        pub fn vibrate(duration: u32) -> Result<(), HapticsError> {
            android::vibrate(duration).map_err(failed_to_vibrate)
        }

        pub fn impact_feedback(style: ImpactFeedbackStyle) -> Result<(), HapticsError> {
            android::impact_feedback(style).map_err(failed_to_vibrate)
        }

        pub fn notification_feedback(style: NotificationFeedbackType) -> Result<(), HapticsError> {
            android::notification_feedback(style).map_err(failed_to_vibrate)
        }

        pub fn selection_feedback() -> Result<(), HapticsError> {
            android::selection_feedback().map_err(failed_to_vibrate)
        }
    } else {
        pub fn vibrate(_duration: u32) -> Result<(), HapticsError> {
            Err(HapticsError::Unsupported)
        }

        pub fn impact_feedback(_style: ImpactFeedbackStyle) -> Result<(), HapticsError> {
            Err(HapticsError::Unsupported)
        }

        pub fn notification_feedback(_style: NotificationFeedbackType) -> Result<(), HapticsError> {
            Err(HapticsError::Unsupported)
        }

        pub fn selection_feedback() -> Result<(), HapticsError> {
            Err(HapticsError::Unsupported)
        }
    }
}
