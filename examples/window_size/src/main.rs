use dioxus::prelude::*;
use dioxus_window::size::{get_window_size, use_window_size};

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    let initial_size = use_signal(|| get_window_size().unwrap());

    let window_size = use_window_size();
    let window_size = window_size().unwrap();

    rsx!(
        div {
            style: "text-align: center;",
            h1 { "↕️ Window Size Utilities ↔️" }
            h3 { "Initial Size" }
            p { "Width: {initial_size().width}" }
            p { "Height: {initial_size().height}" }

            h3 { "Current Size" }
            p { "Width: {window_size.width}" }
            p { "Height: {window_size.height}" }
        }
    )
}
