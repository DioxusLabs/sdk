//! A variety of utility functions and hooks.


cfg_if::cfg_if! {
    if #[cfg(feature = "window_size")] {
        pub mod window;
    }
}
