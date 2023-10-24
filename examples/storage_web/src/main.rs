use dioxus::prelude::*;
use dioxus_std::storage::*;
use std::{collections::HashMap, str::FromStr};

fn main() {
    // init debug tool for WebAssembly
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
    dioxus_web::launch(app);
    // match log::set_boxed_logger(Box::new(simple_logger::SimpleLogger)) {
    //     Ok(_) => log::set_max_level(level.to_level_filter()),
    //     Err(e) => panic!("Failed to initialize logger: {}", e),
    // }
    // dioxus_desktop::launch(app);
}

fn app(cx: Scope) -> Element {
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
    use log::{Record, Metadata};

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