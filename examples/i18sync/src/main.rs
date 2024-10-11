use dioxus::prelude::*;
use dioxus_sdk::i18n::*;
use dioxus_sdk::translate;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, LazyLock, RwLock};

static EN_US: &str = include_str!("./en-US.json");
static ES_ES: &str = include_str!("./es-ES.json");
static IT_IT: &str = include_str!("./it-IT.json");

// Usage for multiwindow desktop application, you can use Mutex instead of RwLock
pub(crate) static I18: LazyLock<Arc<RwLock<UseI18Sync>>> = LazyLock::new(|| {
    Arc::new(RwLock::new(use_i18sync_init("en-US".parse().unwrap(), 
    ("en-US", HashMap::from([("es", vec!["it-IT"])])).into()
        , || {
        let en_us = Language::from_str(EN_US).unwrap();
        let es_es = Language::from_str(ES_ES).unwrap();
        let it_it = Language::from_str(IT_IT).unwrap();
        vec![en_us, es_es, it_it]
    })))
});

fn main() {
    launch(app);
}

fn app() -> Element {

    let dom = VirtualDom::new(NewWindow);
    let cfg = dioxus::desktop::Config::new();
    dioxus::desktop::window().new_window(dom, cfg);
    
    
    use_init_i18n("en-US".parse().unwrap(), 
    ("en-US", HashMap::from([("es", vec!["it-IT"])])).into()
        , || {
        let en_us = Language::from_str(EN_US).unwrap();
        let es_es = Language::from_str(ES_ES).unwrap();
        let it_it = Language::from_str(IT_IT).unwrap();
        vec![en_us, es_es, it_it]
    });

    rsx!(
        Unsync{}
        Sync{}
    )
}

#[allow(non_snake_case)]
fn Unsync() -> Element {
    let mut i18 = use_i18();
    rsx!(
        h1 {"Unsync"}
        div {
            change_language_dropdown{}
            change_language_btn{}
            p { "Translated in selected language: " {translate!(i18, "messages.hello_world")} }
            p { "Fallback due to missing translation: " {translate!(i18, "messages.hello", name: "Dioxus")}  }
            p { "Missing translation and missing in fallback: " {translate!(i18, "unkown_id")}  }
        }
    )
}

#[allow(non_snake_case)]
fn Sync() -> Element {
    let i18_sync = I18.read().unwrap();
    
    rsx!(
        h1 {"Sync"}
        div {
            change_language_dropdown_sync{}
            change_language_btn_sync{}
            p { "Translated in selected language: " {translate!(i18_sync, "messages.hello_world")} }
            p { "Fallback due to missing translation: " {translate!(i18_sync, "messages.hello", name: "Dioxus")}  }
            p { "Missing translation and missing in fallback: " {translate!(i18_sync, "unkown_id")}  }
        }
    )
}

fn change_language_dropdown_sync() -> Element {
    let mut i18 = I18.read().unwrap();
    rsx! {
        select {
            oninput: move |ev| {
                I18.write().unwrap().set_language(ev.value().parse().unwrap())
            },
            {i18.language_list().iter().map(|(id, name, _img)| {
                rsx! { option { 
                    value: id.to_string(), 
                    selected: i18.is_selected(&id),
                    "{name}" 
                }}
            })}
        }
    }
}


fn change_language_btn_sync() -> Element {
    let i18_sync = I18.read().unwrap();
    rsx! {{
        i18_sync.language_list().iter().map(|(id, name, _img)| {
            let id = id.clone();
            rsx! { button {
                    onclick: move |_| {
                        I18.write().unwrap().set_language(id.clone()); 
                    },
                    "{name}"
            }}
        })
    }}
}

fn change_language_btn() -> Element {
    let mut i18 = use_i18();
    rsx! {{
        i18.language_list().iter().map(|(id, name, _img)| {
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
            {i18.language_list().iter().map(|(id, name, _img)| {
                rsx! { option { 
                    value: id.to_string(), 
                    selected: i18.is_selected(id),
                    "{name}" 
                }}
            })}
        }
    }
}

#[allow(non_snake_case)]
fn NewWindow() -> Element {
    use_init_i18n("en-US".parse().unwrap(), 
    ("en-US", HashMap::from([("es", vec!["it-IT"])])).into()
        , || {
        let en_us = Language::from_str(EN_US).unwrap();
        let es_es = Language::from_str(ES_ES).unwrap();
        let it_it = Language::from_str(IT_IT).unwrap();
        vec![en_us, es_es, it_it]
    });

    rsx!(
        Unsync{}
        Sync{}
    )
}



