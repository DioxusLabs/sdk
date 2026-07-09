#![allow(non_snake_case)]

use crate::{ImpactFeedbackStyle, NotificationFeedbackType};
use std::sync::OnceLock;

mod ffi {
    #[cfg(target_os = "android")]
    #[manganis::ffi("android")]
    extern "Kotlin" {
        pub type HapticsPlugin;

        pub fn vibrate(this: &HapticsPlugin, duration: i64);
        pub fn impactFeedback(this: &HapticsPlugin, style: String);
        pub fn notificationFeedback(this: &HapticsPlugin, kind: String);
        pub fn selectionFeedback(this: &HapticsPlugin);
    }
}

static PLUGIN: OnceLock<Result<ffi::HapticsPlugin, String>> = OnceLock::new();

fn plugin() -> Result<&'static ffi::HapticsPlugin, String> {
    PLUGIN
        .get_or_init(ffi::HapticsPlugin::new)
        .as_ref()
        .map_err(Clone::clone)
}

pub fn vibrate(duration: u32) -> Result<(), String> {
    let plugin = plugin()?;
    let _ = ffi::vibrate(plugin, duration as i64)?;
    Ok(())
}

pub fn impact_feedback(style: ImpactFeedbackStyle) -> Result<(), String> {
    let plugin = plugin()?;
    let _ = ffi::impactFeedback(plugin, style.to_string())?;
    Ok(())
}

pub fn notification_feedback(kind: NotificationFeedbackType) -> Result<(), String> {
    let plugin = plugin()?;
    let _ = ffi::notificationFeedback(plugin, kind.to_string())?;
    Ok(())
}

pub fn selection_feedback() -> Result<(), String> {
    let plugin = plugin()?;
    let _ = ffi::selectionFeedback(plugin)?;
    Ok(())
}
