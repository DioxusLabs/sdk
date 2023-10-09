//! # dioxus-storage
//! A library for handling local storage ergonomically in Dioxus
//! ## Usage
//! ```rust
//! use dioxus_storage::use_storage;
//! use dioxus::prelude::*;
//! fn main() {
//!     dioxus_web::launch(app)
//! }
//!
//! fn app(cx: Scope) -> Element {
//!     let num = use_persistent(cx, "count", || 0);
//!     cx.render(rsx! {
//!         div {
//!             button {
//!                 onclick: move |_| {
//!                     num.modify(|num| *num += 1);
//!                 },
//!                 "Increment"
//!             }
//!             div {
//!                 "{*num.read()}"
//!             }
//!         }
//!     })
//! }
//! ```

mod client_storage;
mod persistence;
mod storage;

pub use client_storage::{LocalStorage, SessionStorage};
pub use persistence::{use_persistent, use_singleton_persistent};

#[cfg(not(target_family = "wasm"))]
pub use client_storage::set_dir;
