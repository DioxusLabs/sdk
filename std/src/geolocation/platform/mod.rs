cfg_if::cfg_if! {
    if #[cfg(windows)] {
        mod windows;
        pub use self::windows::*;
    } else if #[cfg(target_family = "wasm")] {
        mod wasm;
        pub use self::wasm::*;
    }
}
