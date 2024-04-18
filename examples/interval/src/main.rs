use dioxus::prelude::*;
use dioxus_sdk::utils::interval::use_interval;
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

    rsx!( p { "{count}" } )
}
