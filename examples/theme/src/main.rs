use dioxus::prelude::*;
use dioxus_window::theme::use_system_theme;

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    let theme = use_system_theme();

    let theme_text = match theme() {
        Ok(theme) => rsx! { h3 { "Your system theme is {theme}." } },
        Err(err) => rsx! { h3 {"Error getting system theme: {err:?}" } },
    };

    rsx!(
        div {
            style: "text-align: center;",
            h1 { "🌗 Dioxus 🚀" }
            {theme_text}
        }

        Other {}
    )
}

#[component]
fn Other() -> Element {
    let theme = use_system_theme();

    let theme_text = match theme() {
        Ok(theme) => rsx! { h3 { "Your system theme x2 is {theme}." } },
        Err(err) => rsx! { h3 {"Error getting system theme: {err:?}" } },
    };

    rsx!(
        div {
            style: "text-align: center;",
            {theme_text}
        }
    )
}
