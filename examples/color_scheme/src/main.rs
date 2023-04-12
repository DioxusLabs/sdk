use dioxus::prelude::*;
use dioxus_std::hooks::use_preferred_color_scheme;

fn main() {
    // init debug tool for WebAssembly
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();

    dioxus_web::launch(app);
}

fn app(cx: Scope) -> Element {
    let color_scheme = use_preferred_color_scheme(cx);
    
    render!(
        div {
            style: "text-align: center;",
            h1 { "🌗 Dioxus 🚀" }
            if let Ok(color_scheme) = color_scheme {
                rsx!(
                    h3 { "You preferred color scheme is {color_scheme:?}." }
                )
            } else {
                rsx!(
                    h3 { "There was an error when reading your preferred color scheme."}
                )
            }
        }
    )
}
