use dioxus::prelude::*;
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
    rsx!( "Home" )
}

#[component]
fn Storage() -> Element {
    let mut count_session = use_singleton_persistent(|| 0);
    let mut count_local = use_synced_storage::<LocalStorage, i32>("synced".to_string(), || 0);

    let mut count_local_human = use_synced_storage::<HumanReadableStorage<i32, LocalStorage>, i32>(
        "synced_human".to_string(),
        || 0,
    );

    // let mut in_memory = use_synced_storage::<MemoryStorage<i32>, i32>("memory".to_string(), || 0);

    rsx!(
        div {
            button {
                onclick: move |_| {
                    *count_session.write() += 1;
                },
                "Click me!"
            }
            "I persist for the current session. Clicked {count_session} times."
        }
        div {
            button {
                onclick: move |_| {
                    *count_local.write() += 1;
                },
                "Click me!"
            }
            "I persist across all sessions. Clicked {count_local} times."
        }
        div {
            button {
                onclick: move |_| {
                    *count_local_human.write() += 1;
                },
                "Click me!"
            }
            "I persist a human readable value across all sessions. Clicked {count_local_human} times."
        }
    )

    //   div {
    //         button {
    //             onclick: move |_| {
    //                 *in_memory.write() += 1;
    //             },
    //             "Click me!"
    //         }
    //         "I persist a value without encoding, in memory. Clicked {in_memory} times."
    //     }
}

// Define a "human readable" storage format which is pretty printed JSON instead of a compressed binary format.
type HumanReadableStorage<T, Storage> = LayeredStorage<T, Storage, HumanReadableEncoding>;

#[derive(Clone)]
struct HumanReadableEncoding;

impl<T: Serialize + DeserializeOwned> StorageEncoder<T> for HumanReadableEncoding {
    type EncodedValue = String;

    fn deserialize(loaded: &Self::EncodedValue) -> T {
        let parsed: Result<T, serde_json::Error> = serde_json::from_str(loaded);
        // This design probably needs an error handling policy better than panic.
        parsed.unwrap()
    }

    fn serialize(value: &T) -> Self::EncodedValue {
        serde_json::to_string_pretty(value).unwrap()
    }
}

// // In memory cloned format
// type MemoryStorage<T> = LayeredStorage<T, InMemoryEncoder, InMemoryEncoder>;

// #[derive(Clone)]
// pub struct InMemoryEncoder;

// impl<T: Clone + Any + Send> StorageEncoder<T> for InMemoryEncoder {
//     type EncodedValue = Arc<Mutex<dyn Any + Send>>;

//     fn deserialize(loaded: &Self::EncodedValue) -> T {
//         let x = loaded.lock().unwrap();
//         // TODO: handle errors
//         x.downcast_ref::<T>().cloned().unwrap()
//     }

//     fn serialize(value: &T) -> Self::EncodedValue {
//         Arc::new(Mutex::new(value.clone()))
//     }
// }

// impl StoragePersistence for InMemoryEncoder {
//     type Key = String;
//     type Value = Option<Arc<Mutex<dyn Any + Send>>>;

//     fn load(key: &Self::Key) -> Self::Value {
//         IN_MEMORY_STORE.read().unwrap().get(key).map(|v| v.clone())
//     }

//     fn store<T: 'static + Clone + Send>(key: &Self::Key, value: &Self::Value, unencoded: &T) {
//         let mut map = IN_MEMORY_STORE.write().unwrap();
//         if let Some(v) = value {
//             map.insert(key.clone(), v.clone());
//         } else {
//             map.remove(key);
//         }
//     }
// }

// impl<S, T> StorageSubscriber<T, S> for InMemoryEncoder {
//     fn subscribe(key: &<S as StorageBacking>::Key) -> Receiver<StorageChannelPayload> {
//         todo!()
//     }

//     fn unsubscribe(key: &<S as StorageBacking>::Key) {
//         todo!()
//     }
// }

// static IN_MEMORY_STORE: LazyLock<RwLock<HashMap<String, Arc<Mutex<dyn Any + Send>>>>> =
//     LazyLock::new(|| RwLock::default());
