use dioxus::prelude::*;
use dioxus_sdk::utils::timing::{use_debounce, use_interval};
use std::time::Duration;

fn main() {
    // init debug tool for WebAssembly
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();

    launch(app);
}

fn app() -> Element {
    let mut count = use_signal(|| 0);

    use_interval(Duration::from_millis(100), move || {
        count += 1;
    });

    let mut debounce = use_debounce(Duration::from_millis(2000), move |text| {
        println!("{text}");
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
