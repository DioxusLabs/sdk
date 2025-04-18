# Dioxus Storage
Local and persistent storage utilities for Dioxus.

### Features:
- [x] Local Storage
- [x] Persistent Storage

## Usage
Add `dioxus-storage` to your `Cargo.toml`:
```toml
[dependencies]
dioxus_storage = "0.1"
```

Example:
```rs
use dioxus_storage::use_persistent;
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

### Dioxus Compatibility
This table represents the compatibility between this crate and Dioxus versions.
The crate version supports a Dioxus version up until the next crate version in the table.

E.g. if crate version `0.1` supported Dioxus `0.6` and crate version `0.4` supported Dioxus `0.7`, crate versions `0.1`, `0.2`, and `0.3` would support Dioxus `0.6`.

| Crate Version | Dioxus Version |
| ------------- | -------------- |
| 0.1           | 0.6            |