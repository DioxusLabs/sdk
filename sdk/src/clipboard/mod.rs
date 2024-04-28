cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        mod use_clipboard;
        pub use use_clipboard::*;
    } else {
        compile_error!("the `clipboard` feature is only available on desktop targets");
    }
}
