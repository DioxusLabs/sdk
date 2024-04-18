//#![warn(missing_debug_implementations, missing_docs)]

cfg_if::cfg_if! {
    if #[cfg(feature = "color_scheme")] {
        pub mod color_scheme;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "geolocation")] {
        pub mod geolocation;
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(feature = "channel", feature = "use_window_size"))] {
        pub mod utils;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "i18n")] {
        pub mod i18n;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "clipboard")] {
        pub mod clipboard;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "storage")] {
        pub mod storage;
    }
}

