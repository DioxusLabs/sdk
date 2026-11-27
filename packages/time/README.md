# Dioxus Time

Deprecated: these timer hooks now live in Dioxus as [`dioxus-timers`](https://docs.rs/dioxus-timers). Enable the `timers` feature of `dioxus` and use `dioxus::timers` instead:

```toml
[dependencies]
dioxus = { version = "0.8", features = ["timers"] }
```

```rust
use dioxus::timers::{use_debounce, use_interval, use_timeout, sleep};
```

This crate re-exports `dioxus-timers` so existing code keeps working.
