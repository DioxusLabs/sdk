use crate::{ImpactFeedbackStyle, NotificationFeedbackType};

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

fn plugin() -> Result<ffi::HapticsPlugin, &'static str> {
    ffi::HapticsPlugin::new()
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
