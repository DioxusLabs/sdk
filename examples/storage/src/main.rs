use dioxus::prelude::*;
use dioxus_std::storage::*;
use std::{collections::HashMap, str::FromStr};

fn main() {
    // init debug tool for WebAssembly
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();

    dioxus_web::launch(app);
}

fn app(cx: Scope) -> Element {
    let count = use_singleton_persistent(cx, || 0);

    render!(
        button {
            onclick: move |_| {
                count.set(count.get() + 1);
            },
            "Click me!"
        },
        "Clicked {count} times"
    )
}
