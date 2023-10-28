use dioxus::prelude::*;
use dioxus_router::prelude::*;
use dioxus_std::storage::*;

fn main() {
    match log::set_boxed_logger(Box::new(simple_logger::SimpleLogger)) {
        Ok(_) => log::set_max_level(log::LevelFilter::Info),
        Err(e) => panic!("Failed to initialize logger: {}", e),
    }
    dioxus_std::storage::set_dir!();
    dioxus_desktop::launch(App);
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
    let window = dioxus_desktop::use_window(cx);

    render! {
        div {
            Outlet::<Route> { }

            p {
                "----"
            }

            div {
                button {
                    onclick: move |_| {
                        let dom = VirtualDom::new(App);
                        window.new_window(dom, Default::default());
                    },
                    "New Window"
                }
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

mod simple_logger {
    use log::{Metadata, Record};

    pub struct SimpleLogger;

    impl log::Log for SimpleLogger {
        fn enabled(&self, metadata: &Metadata) -> bool {
            metadata.level() <= log::max_level()
        }

        fn log(&self, record: &Record) {
            if self.enabled(record.metadata()) {
                println!("{} - {}", record.level(), record.args());
            }
        }

        fn flush(&self) {}
    }
}
