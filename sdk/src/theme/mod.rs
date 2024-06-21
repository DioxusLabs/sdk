//! Interact with the system theme.

cfg_if::cfg_if! {
    if #[cfg(any(target_family = "wasm", target_os = "windows", target_os = "macos"))] {
        mod system_theme;
        pub use system_theme::*;
    } else {
        compile_error!("the `color_scheme` feature is only available on wasm, windows, and macos targets");
    }
}
