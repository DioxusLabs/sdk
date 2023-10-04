cfg_if::cfg_if! {
    if #[cfg(target_family = "wasm")] {
        pub mod web;
        pub use web::*;
    } else {
        pub mod fs;
        pub use fs::*;
    }
}