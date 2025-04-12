//#![warn(missing_debug_implementations, missing_docs)]

cfg_if::cfg_if! {
    if #[cfg(feature = "system_theme")] {
        pub mod theme;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "geolocation")] {
        pub mod geolocation;
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(feature = "channel", feature = "scroll", feature = "window_size", feature = "timing"))] {
        pub mod utils;
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

cfg_if::cfg_if! {
    if #[cfg(feature = "notifications")] {
        pub mod notification;
    }
}
