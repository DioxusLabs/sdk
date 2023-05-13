//! Useful hooks that integrate into your Dioxus app seamlessly. Included with these hooks are Dioxus-friendly methods of accessing the abstractions in the [crate::library] module.
cfg_if::cfg_if! {
    if #[cfg(feature = "use_preferred_color_scheme")] {
        pub mod use_preferred_color_scheme;
        pub use use_preferred_color_scheme::*;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "geolocation")] {
        pub mod use_geolocation;
        pub use use_geolocation::*;
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(feature = "use_rw", feature = "geolocation"))] {
        pub mod use_rw;
        pub use use_rw::*;
    }
}
