# Dioxus Time
Timing utilities and hooks for Dioxus.

### Features:
- [x] Intervals
- [x] Debounces
- [ ] Timeouts

## Usage
Add `dioxus-sdk-time` to your `Cargo.toml`:
```toml
[dependencies]
dioxus-sdk-time = "0.1"
```

Example:
```rs
use dioxus::{logger::tracing::info, prelude::*};
use dioxus_sdk_time::{use_debounce, use_interval};
use std::time::Duration;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut count = use_signal(|| 0);

    // Increment count every second.
    use_interval(Duration::from_secs(1), move || count += 1);

    // Reset count after 2 seconds of the latest action call.
    let mut debounce = use_debounce(Duration::from_millis(2000), move |text| {
        info!("{text}");
        count.set(0);
    });

    rsx! {
        p { "{count}" },
        button {
            // Trigger the debounce.
            onclick: move |_| debounce.action("button was clicked"),
            "Reset the counter! (2 second debounce)"
        }
    }
}

```
