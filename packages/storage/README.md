# Dioxus Storage
Local and persistent storage utilities for Dioxus.

### Features:
- [x] Local Storage
- [x] Persistent Storage

## Usage
Add `dioxus-storage` to your `Cargo.toml`:
```toml
[dependencies]
dioxus_sdk_storage = "0.1"
```

Example:
```rs
use dioxus_sdk_storage::use_persistent;
use dioxus::prelude::*;

#[component]
fn App() -> Element {
    let mut num = use_persistent("count", || 0);
    rsx! {
        div {
            button {
                onclick: move |_| {
                    *num.write() += 1;
                },
                "Increment"
            }
            div {
                "{*num.read()}"
            }
        }
    }
}
```
