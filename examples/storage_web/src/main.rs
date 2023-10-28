use dioxus::prelude::*;
use dioxus_router::prelude::*;
use dioxus_std::storage::*;

fn main() {
    // init debug tool for WebAssembly
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    dioxus_web::launch(App);
}

#[component]
fn App(cx: Scope) -> Element {
    render! {
        Router::<Route> {}
    }
}

#[derive(Routable, Clone)]
#[rustfmt::skip]
enum Route {
    #[layout(Footer)]
        #[route("/")]
        Page1 {},
        #[route("/page2")]
        Page2 {},
}

#[component]
fn Footer(cx: Scope) -> Element {
    render! {
        div {
            Outlet::<Route> { }

            p {
                "----"
            }

            nav {
                ul {
                    li { Link { to: Route::Page1 {}, "Page1" } }
                    li { Link { to: Route::Page2 {}, "Page2" } }
                }
            }
        }
    }
}

#[component]
fn Page1(cx: Scope) -> Element {
    render!("Home")
}

#[component]
fn Page2(cx: Scope) -> Element {
    let count_session = use_singleton_persistent(cx, || 0);
    let count_local = use_synced_storage::<LocalStorage, i32>(cx, "synced".to_string(), || 0);

    render!(
        div {
            button {
                onclick: move |_| {
                    *count_session.write() += 1;
                },
                "Click me!"
            },
            "I persist for the current session. Clicked {count_session} times"
        }
        div {
            button {
                onclick: move |_| {
                    *count_local.write() += 1;
                },
                "Click me!"
            },
            "I persist across all sessions. Clicked {count_local} times"
        }
    )
}
