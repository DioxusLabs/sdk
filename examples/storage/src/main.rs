use dioxus::prelude::*;
use dioxus_storage::*;

fn main() {
    dioxus_storage::set_dir!();
    launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[derive(Routable, Clone)]
#[rustfmt::skip]
enum Route {
    #[layout(Footer)]
        #[route("/")]
        Home {},
        #[route("/storage")]
        Storage {},
}

#[component]
fn Footer() -> Element {
    let new_window = {
        #[cfg(feature = "desktop")]
        {
            let window = dioxus::desktop::use_window();
            rsx! {
                div {
                    button {
                        onclick: move |_| {
                            let dom = VirtualDom::new(App);
                            window.new_window(dom, Default::default());
                        },
                        "New Window"
                    }
                }
            }
        }
        #[cfg(not(feature = "desktop"))]
        {
            rsx! {
                div {}
            }
        }
    };

    rsx! {
        div {
            Outlet::<Route> { }

            p {
                "----"
            }

            {new_window}

            nav {
                ul {
                    li { Link { to: Route::Home {}, "Home" } }
                    li { Link { to: Route::Storage {}, "Storage" } }
                }
            }
        }
    }
}

#[component]
fn Home() -> Element {
    rsx!("Home")
}

#[component]
fn Storage() -> Element {
    let mut count_session = use_singleton_persistent(|| 0);
    let mut count_local = use_synced_storage::<LocalStorage, i32>("synced".to_string(), || 0);

    rsx!(
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
