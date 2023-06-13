//#![warn(missing_debug_implementations, missing_docs)]

//! Useful hooks that integrate into your Dioxus app seamlessly. Included with these hooks are Dioxus-friendly methods of accessing the abstractions in the [crate::library] module.
cfg_if::cfg_if! {
    if #[cfg(feature = "color_scheme")] {
        pub mod color_scheme;
        pub use use_preferred_color_scheme::*;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "geolocation")] {
        pub mod geolocation;
        pub use geolocation::*;
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(feature = "utils"))] {
        pub mod utils;
        pub use utils::*;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "i18n")] {
        pub mod i18n;
        pub use i18n::*;
    }
}
