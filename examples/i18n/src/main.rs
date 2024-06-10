use dioxus::prelude::*;
use dioxus_sdk::i18n::*;
use dioxus_sdk::translate;
use std::str::FromStr;

fn main() {
    // init debug tool for WebAssembly
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();

    launch(app);
}

static EN_US: &str = include_str!("./en-US.json");
static ES_ES: &str = include_str!("./es-ES.json");

fn change_language_btn() -> Element {
    let mut i18 = use_i18();
    rsx! {{
        (i18.language_list().iter()).map(|(id, name)| {
            let id = id.clone();
            rsx! { button {
                    onclick: move |_| { i18.set_language(id.clone()); },
                    "{name}"
            }}
        })
    }}
}

fn change_language_dropdown() -> Element {
    let mut i18 = use_i18();
    rsx! {
        select {
            oninput: move |ev| {
                i18.set_language(ev.value().parse().unwrap())
            },
            {(i18.language_list().iter()).map(|(id, name)| {
                rsx! { option { value: id.to_string(), "{name}" }}
            })}
        }
    }
}

#[allow(non_snake_case)]
fn Body() -> Element {
    let mut i18 = use_i18();
    rsx!(
        change_language_dropdown{}
        change_language_btn{}
        p { {translate!(i18, "messages.hello_world")} }
        p { {translate!(i18, "messages.hello", name: "Dioxus")}  }
    )
}

fn app() -> Element {
    use_init_i18n("en-US".parse().unwrap(), "en-US".parse().unwrap(), || {
        let en_us = Language::from_str(EN_US).unwrap();
        let es_es = Language::from_str(ES_ES).unwrap();
        vec![en_us, es_es]
    });

    rsx!(Body {})
}
