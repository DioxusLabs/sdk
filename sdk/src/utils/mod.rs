//! A variety of utility functions and hooks.

cfg_if::cfg_if! {
    if #[cfg(feature = "channel")] {
        pub mod channel;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "window_size")] {
        pub mod window;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "timing")] {
        pub mod timing;
    }
}
