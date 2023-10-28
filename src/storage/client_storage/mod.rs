cfg_if::cfg_if! {
    if #[cfg(target_family = "wasm")] {
        pub mod web;
        pub use web::*;
    } else {
        pub mod fs;
        pub use fs::*;
        pub mod memory;
        pub use memory::SessionStorage;
    }
}
