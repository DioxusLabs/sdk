use dioxus::prelude::*;
use dioxus_std::*;
use std::{collections::HashMap, str::FromStr};

fn main() {
    // init debug tool for WebAssembly
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();

    dioxus_web::launch(app);
}

static EN_US: &str = include_str!("./en-US.json");
static ES_ES: &str = include_str!("./es-ES.json");

#[allow(non_snake_case)]
fn Body(cx: Scope) -> Element {
    let i18 = use_i18(cx);

    let change_to_english = move |_| i18.set_language("en-US".parse().unwrap());
    let change_to_spanish = move |_| i18.set_language("es-ES".parse().unwrap());

    render!(
        button {
            onclick: change_to_english,
            label {
                "English"
            }
        }
        button {
            onclick: change_to_spanish,
            label {
                "Spanish"
            }
        }
        p { translate!(i18, "messages.hello_world") }
        p { translate!(i18, "messages.hello", name: "Dioxus")  }
    )
}

fn app(cx: Scope) -> Element {
    use_init_i18n(cx, "en-US".parse().unwrap(), "en-US".parse().unwrap(), || {
        let en_us = Language::from_str(EN_US).unwrap();
        let es_es = Language::from_str(ES_ES).unwrap();
        vec![en_us, es_es]
    });

    render!(Body {})
}