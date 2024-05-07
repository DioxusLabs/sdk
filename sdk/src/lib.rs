//#![warn(missing_debug_implementations, missing_docs)]

cfg_if::cfg_if! {
    if #[cfg(feature = "system_theme")] {
        pub mod system_theme;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "geolocation")] {
        pub mod geolocation;
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(feature = "channel", feature = "window_size", feature = "timing"))] {
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
