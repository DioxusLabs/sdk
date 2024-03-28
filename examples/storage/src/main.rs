use dioxus::prelude::*;
use dioxus_router::prelude::*;
use dioxus_std::storage::*;

fn main() {
    dioxus_std::storage::set_dir!();
    launch(app);
}

fn app() -> Element {
    rsx! {
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
fn Footer() -> Element {
    
    let new_window = {
        #[cfg(feature = "desktop")]
        {
            let window = dioxus::desktop::use_window();
            rsx! {
                div {
                    button {
                        onclick: move |_| {
                            let dom = VirtualDom::new(app);
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
                    li { Link { to: Route::Page1 {}, "Page1" } }
                    li { Link { to: Route::Page2 {}, "Page2" } }
                }
            }
        }
    }
}

#[component]
fn Page1() -> Element {
    rsx!("Home")
}

#[component]
fn Page2() -> Element {
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
