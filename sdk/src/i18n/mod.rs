//! Provide translations for your app.
mod tanslate;
mod i18n;
mod i18n_context;

pub use i18n_context::{use_i18, use_init_i18n};
pub use i18n::Language;

mod i18n_static;
pub use i18n_static::{use_i18sync_init, UseI18Sync};