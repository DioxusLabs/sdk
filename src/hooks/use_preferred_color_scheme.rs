use dioxus::prelude::ScopeState;
use std::sync::Once;
use wasm_bindgen::{prelude::Closure, JsCast};

#[derive(Debug, Clone)]
pub enum PreferredColorScheme {
    Light,
    Dark,
    Err(String),
}

static INIT: Once = Once::new();

pub fn use_preferred_color_scheme(cx: &ScopeState) -> PreferredColorScheme {
    // This code is kinda messy..
    let window = match web_sys::window() {
        Some(w) => w,
        None => {
            return PreferredColorScheme::Err(
                "not running in wasm context: window doesn't exist".to_string(),
            )
        }
    };

    let media_query = match window.match_media("(prefers-color-scheme: dark)") {
        Ok(opt) => match opt {
            Some(m) => m,
            None => {
                return PreferredColorScheme::Err(
                    "failed to determine preferred scheme".to_string(),
                )
            }
        },
        Err(e) => {
            return PreferredColorScheme::Err(e.as_string().unwrap_or(
                "failed to determine preferred scheme and couldn't retrieve error".to_string(),
            ))
        }
    };

    let update_callback = cx.schedule_update();

    // Create closure that listens to the event of matchMedia and calls write to scheme
    INIT.call_once(|| {
        let listener = Closure::<dyn Fn()>::new(move || {
            update_callback();
        });
        media_query.set_onchange(Some(listener.as_ref().unchecked_ref()));
    });

    determine_scheme(media_query.matches())
}

fn determine_scheme(value: bool) -> PreferredColorScheme {
    match value {
        true => PreferredColorScheme::Dark,
        false => PreferredColorScheme::Light,
    }
}
