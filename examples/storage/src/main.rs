use dioxus::{logger::tracing, prelude::*};
use dioxus_storage::*;

use serde::{de::DeserializeOwned, Serialize};

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
            Outlet::<Route> {}

            p { "----" }

            {new_window}

            nav {
                ul {
                    li {
                        Link { to: Route::Home {}, "Home" }
                    }
                    li {
                        Link { to: Route::Storage {}, "Storage" }
                    }
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
    tracing::debug!("-- Start --");

    // TODO: maybe this should sync? (It does not currently)
    // Uses default encoder and LocalStorage implicitly.
    let mut count_persistent = use_persistent("persistent".to_string(), || 0);

    // Uses session storage with the default encoder.
    let mut count_session = use_storage::<SessionStorage, i32>("session".to_string(), || 0);

    // Uses local storage with the default encoder.
    let mut count_local = use_synced_storage::<LocalStorage, i32>("local".to_string(), || 0);

    // TODO: this does not sync in web
    // Uses LocalStorage with our custom human readable encoder
    let mut count_local_human = use_synced_storage::<HumanReadableStorage<i32, LocalStorage>, i32>(
        "local_human".to_string(),
        || 0,
    );

    rsx!(
        div {
            button {
                onclick: move |_| {
                    *count_persistent.write() += 1;
                },
                "Click me!"
            }
            "Persisted to local storage (but not synced): Clicked {count_persistent} times."
        }
        div {
            button {
                onclick: move |_| {
                    *count_session.write() += 1;
                },
                "Click me!"
            }
            "Session: Clicked {count_session} times."
        }
        div {
            button {
                onclick: move |_| {
                    *count_local.write() += 1;
                },
                "Click me!"
            }
            "Local: Clicked {count_local} times."
        }
        div {
            button {
                onclick: move |_| {
                    *count_local_human.write() += 1;
                },
                "Click me!"
            }
            "Human readable local: Clicked {count_local_human} times."
        }
    )
}

// Define a "human readable" storage format which is pretty printed JSON instead of a compressed binary format.
//
// `Storage` must have `Value=Option<string>` for this to work as that is what the encoder encodes to.
type HumanReadableStorage<T, Storage> = LayeredStorage<T, Storage, HumanReadableEncoding>;

#[derive(Clone)]
struct HumanReadableEncoding;

impl<T: Serialize + DeserializeOwned> StorageEncoder<T> for HumanReadableEncoding {
    type EncodedValue = String;
    type DecodeError = serde_json::Error;

    fn deserialize(loaded: &Self::EncodedValue) -> Result<T, Self::DecodeError> {
        serde_json::from_str(loaded)
    }

    fn serialize(value: &T) -> Self::EncodedValue {
        serde_json::to_string_pretty(value).unwrap()
    }
}
