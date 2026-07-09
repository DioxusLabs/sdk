use dioxus::prelude::*;
use dioxus_sdk_haptics::{
    impact_feedback, notification_feedback, selection_feedback, vibrate, ImpactFeedbackStyle,
    NotificationFeedbackType,
};

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    let status = use_signal(|| "Tap a button to test haptics.".to_string());

    rsx! {
        div {
            style: "max-width: 720px; margin: 0 auto; padding: 24px; font-family: sans-serif;",
            h1 { "Dioxus Haptics Example" }
            p { "Run this on an mobile device to exercise the haptics bridge." }
            p { strong { "Status: " } "{status}" }

            section {
                style: "margin-top: 24px;",
                h2 { "Vibrate" }
                div {
                    style: "display: flex; gap: 12px; flex-wrap: wrap;",
                    button {
                        onclick: move |_| run_action(status, "vibrate 25ms", map_result(vibrate(25))),
                        "25 ms"
                    }
                    button {
                        onclick: move |_| run_action(status, "vibrate 75ms", map_result(vibrate(75))),
                        "75 ms"
                    }
                    button {
                        onclick: move |_| run_action(status, "vibrate 150ms", map_result(vibrate(150))),
                        "150 ms"
                    }
                }
            }

            section {
                style: "margin-top: 24px;",
                h2 { "Impact Feedback" }
                div {
                    style: "display: flex; gap: 12px; flex-wrap: wrap;",
                    button {
                        onclick: move |_| run_action(status, "impact light", map_result(impact_feedback(ImpactFeedbackStyle::Light))),
                        "Light"
                    }
                    button {
                        onclick: move |_| run_action(status, "impact medium", map_result(impact_feedback(ImpactFeedbackStyle::Medium))),
                        "Medium"
                    }
                    button {
                        onclick: move |_| run_action(status, "impact heavy", map_result(impact_feedback(ImpactFeedbackStyle::Heavy))),
                        "Heavy"
                    }
                    button {
                        onclick: move |_| run_action(status, "impact soft", map_result(impact_feedback(ImpactFeedbackStyle::Soft))),
                        "Soft"
                    }
                    button {
                        onclick: move |_| run_action(status, "impact rigid", map_result(impact_feedback(ImpactFeedbackStyle::Rigid))),
                        "Rigid"
                    }
                }
            }

            section {
                style: "margin-top: 24px;",
                h2 { "Notification Feedback" }
                div {
                    style: "display: flex; gap: 12px; flex-wrap: wrap;",
                    button {
                        onclick: move |_| run_action(status, "notification success", map_result(notification_feedback(NotificationFeedbackType::Success))),
                        "Success"
                    }
                    button {
                        onclick: move |_| run_action(status, "notification warning", map_result(notification_feedback(NotificationFeedbackType::Warning))),
                        "Warning"
                    }
                    button {
                        onclick: move |_| run_action(status, "notification error", map_result(notification_feedback(NotificationFeedbackType::Error))),
                        "Error"
                    }
                }
            }

            section {
                style: "margin-top: 24px;",
                h2 { "Selection" }
                button {
                    onclick: move |_| run_action(status, "selection feedback", map_result(selection_feedback())),
                    "Selection"
                }
            }
        }
    }
}

fn run_action(mut status: Signal<String>, label: &'static str, result: Result<(), String>) {
    match result {
        Ok(()) => status.set(format!("Triggered {label}.")),
        Err(err) => status.set(format!("{label} failed: {err}")),
    }
}

fn map_result(result: Result<(), dioxus_sdk_haptics::HapticsError>) -> Result<(), String> {
    result.map_err(|err| err.to_string())
}
