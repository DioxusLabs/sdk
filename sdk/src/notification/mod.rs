//! Send desktop notifications.

cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        mod desktop;
        pub use desktop::*;
    } else {
        compile_error!("the `notification` feature is only available on desktop targets");
    }
}
