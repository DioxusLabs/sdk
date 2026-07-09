# Dioxus Haptics
Haptics utilities for Dioxus.

Heavily inspired by [tauri-plugin-haptics](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/haptics)

### Supports
- [ ] Web
- [ ] Windows
- [ ] Mac
- [ ] Linux
- [x] Android
- [x] iOs

## Usage
Add `dioxus-sdk-haptics` to your `Cargo.toml`:
```toml
[dependencies]
dioxus-sdk-haptics = "0.7"
```

Example:
```rs
use dioxus::prelude::*;
use dioxus_sdk_haptics::vibrate;

#[component]
fn App() -> Element {
    let mut status = use_signal(|| "Tap the button to vibrate.".to_string());

    rsx! {
        div {
            p { strong { "Status: " } "{status}" }
            button {
                onclick: move |_| match vibrate(25).map_err(|err| err.to_string()){
                    Ok(()) => status.set(format!("Triggered 25ms.")),
                    Err(err) => status.set(format!("25ms failed: {err}")),
                },
                "25 ms"
            }
        }
    }
}
```
