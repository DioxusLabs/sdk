cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        pub mod notification;
    } else {
        compile_error!("the `notification` feature is only available on desktop targets");
    }
}