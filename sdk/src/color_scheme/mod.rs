cfg_if::cfg_if! {
    if #[cfg(target_family = "wasm")] {
        mod use_preferred_color_scheme;
        pub use use_preferred_color_scheme::*;
    } else {
        compile_error!("the `color_scheme` feature is only available on wasm targets");
    }
}
