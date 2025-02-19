#[allow(clippy::needless_doctest_main)]
/// Set the directory where the storage files are located on non-wasm targets.
///
/// ```rust
/// use dioxus_storage::set_dir;
///
/// fn main(){
///     // set the directory to the default location
///     set_dir!();
/// }
/// ```
/// ```rust
/// use dioxus_storage::set_dir;
///
/// fn main(){
///     // set the directory to a custom location
///     set_dir!("path/to/dir");
/// }
/// ```
#[macro_export]
macro_rules! set_dir {
    () => {
        #[cfg(not(target_family = "wasm"))]
        $crate::set_dir_name(env!("CARGO_PKG_NAME"))
    };
    ($path: literal) => {
        #[cfg(not(target_family = "wasm"))]
        $crate::set_directory(std::path::PathBuf::from($path))
    };
}

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
