use crate::{ImpactFeedbackStyle, NotificationFeedbackType};
use std::sync::OnceLock;

mod ffi {
    #[allow(non_snake_case)]
    #[manganis::ffi("/ios")]
    unsafe extern "Swift" {
        pub type HapticsPlugin;

        pub fn vibrate(this: &HapticsPlugin, duration: i64);
        pub fn impact_feedback(this: &HapticsPlugin, style: &str);
        pub fn notification_feedback(this: &HapticsPlugin, kind: &str);
        pub fn selection_feedback(this: &HapticsPlugin);
    }
}

static PLUGIN: OnceLock<Result<ffi::HapticsPlugin, &'static str>> = OnceLock::new();

fn plugin() -> Result<&'static ffi::HapticsPlugin, &'static str> {
    PLUGIN
        .get_or_init(ffi::HapticsPlugin::new)
        .as_ref()
        .map_err(|err| *err)
}

pub fn vibrate(duration: u32) -> Result<(), &'static str> {
    let plugin = plugin()?;
    ffi::vibrate(&plugin, i64::from(duration))
}

pub fn impact_feedback(style: ImpactFeedbackStyle) -> Result<(), &'static str> {
    let plugin = plugin()?;
    ffi::impact_feedback(&plugin, &style.to_string())
}

pub fn notification_feedback(kind: NotificationFeedbackType) -> Result<(), &'static str> {
    let plugin = plugin()?;
    ffi::notification_feedback(&plugin, &kind.to_string())
}

pub fn selection_feedback() -> Result<(), &'static str> {
    let plugin = plugin()?;
    ffi::selection_feedback(&plugin)
}
