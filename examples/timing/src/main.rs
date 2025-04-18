use dioxus::{logger::tracing::info, prelude::*};
use dioxus_time::{use_debounce, use_interval, use_timeout};
use std::time::Duration;

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    let mut count = use_signal(|| 0);

    // using `use_interval`, we increment the count by 1 every second.
    use_interval(Duration::from_secs(1), move |()| {
        count += 1;
    });

    // using `use_debounce`, we reset the counter after 2 seconds since the last button click.
    let mut debounce = use_debounce(Duration::from_secs(2), move |text| {
        info!("{text}");
        count.set(0);
    });

    // using `use_timeout`, we increase a the counter 2 seconds after every trigger.
    let mut timeout_count = use_signal(|| 0);
    let timeout = use_timeout(Duration::from_secs(2), move |()| {
        timeout_count += 1;
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
        button {
            onclick: move |_| { timeout.action(()); },
            "Trigger Timeout: {timeout_count}",
        }
    }
}
