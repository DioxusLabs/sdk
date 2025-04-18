use dioxus::{
    logger::tracing::{info, Level},
    prelude::*,
};
use dioxus_util::scroll::use_root_scroll;

fn main() {
    dioxus::logger::init(Level::TRACE).unwrap();
    launch(App);
}

#[component]
fn App() -> Element {
    let random_text = "This is some random repeating text. ".repeat(1000);

    let scroll_metrics = use_root_scroll();
    use_effect(move || {
        let scroll_metrics = scroll_metrics();
        let distance_from_bottom = scroll_metrics.scroll_height
            - (scroll_metrics.scroll_top + scroll_metrics.client_height);
        info!("Distance from bottom: {}", distance_from_bottom);
        let scroll_percentage = (scroll_metrics.scroll_top + scroll_metrics.client_height)
            / scroll_metrics.scroll_height;
        info!("Scroll percentage: {}", scroll_percentage);
    });

    rsx! {
        div { style: "text-align: center; padding: 20px; font-family: sans-serif;",
            h1 { "Random Text" }
            div { style: "margin: 20px; padding: 15px; border: 1px solid #ccc; border-radius: 5px;",
                p { "{random_text}" }
            }
        }
    }
}
