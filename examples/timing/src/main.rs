use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use dioxus_sdk::utils::timing::{use_debounce, use_interval};
use std::time::Duration;

fn main() {
    dioxus_logger::init(Level::INFO).expect("logger failed to init");
    launch(app);
}

fn app() -> Element {
    let mut count = use_signal(|| 0);

    // using `use_interval`, we increment the count by 1 every 100 milliseconds.
    use_interval(Duration::from_millis(1000), move || {
        count += 1;
    });

    // using `use_debounce`, we reset the counter after 2 seconds since the last button click.
    let mut debounce = use_debounce(Duration::from_millis(2000), move |text| {
        info!("{text}");
        count.set(0);
    });

    rsx! {
        p { "{count}" },
        button {
            onclick: move |_| {
                // Reset the counter after 2 seconds pass since the last click.
                debounce.action("button was clicked");
            },
            "Reset the counter! (2 second debounce)"
        }
    }
}
