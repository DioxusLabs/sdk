//! Useful hooks that integrate into your Dioxus app seamlessly. Included with these hooks are Dioxus-friendly methods of accessing the abstractions in the [crate::library] module.

#[cfg(feature = "use_preferred_color_scheme")]
pub mod use_preferred_color_scheme;
#[cfg(feature = "use_preferred_color_scheme")]
pub use use_preferred_color_scheme::*;
