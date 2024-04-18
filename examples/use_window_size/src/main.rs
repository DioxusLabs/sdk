use dioxus::prelude::*;
use dioxus_sdk::utils::use_window_size;

fn main() {
    // init debug tool for WebAssembly
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();

    launch(app);
}

fn app() -> Element {
    let window_size = use_window_size();

    rsx!(
        div { style: "text-align: center;",
            h1 { "🌗 Dioxus 🚀" }
            p { "Width: {window_size.0}" }
            p { "Height: {window_size.1}" }
        }
    )
}
