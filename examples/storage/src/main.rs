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
    let count1 = use_singleton_persistent(cx, || 0);
    let count2 = use_singleton_persistent(cx, || 0);

    render!(
        div {
            button {
                onclick: move |_| {
                    count1.set(count1.get() + 1);
                },
                "Click me!"
            },
            "Clicked {count1} times"
        }
        div {
            button {
                onclick: move |_| {
                    count2.set(count2.get() + 1);
                },
                "Click me!"
            },
            "Clicked {count2} times"
        }
    )
}
