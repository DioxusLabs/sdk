use serde::{Deserialize, Serialize};

use std::{
    error::Error,
    fmt::{self, Display},
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpactFeedbackStyle {
    Light,
    #[default]
    Medium,
    Heavy,
    Soft,
    Rigid,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationFeedbackType {
    #[default]
    Success,
    Warning,
    Error,
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

pub fn vibrate(duration: u32) -> Result<(), HapticsError> {
    Ok(())
}

pub fn impact_feedback(style: ImpactFeedbackStyle) -> Result<(), HapticsError> {
    Ok(())
}

pub fn notification_feedback(style: NotificationFeedbackType) -> Result<(), HapticsError> {
    Ok(())
}

pub fn selection_feedback() -> Result<(), HapticsError> {
    Ok(())
}
