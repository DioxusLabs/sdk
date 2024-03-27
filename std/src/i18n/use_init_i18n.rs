use dioxus::prelude::*;
use unic_langid::LanguageIdentifier;

use super::use_i18n::Language;

pub struct UseInitI18Data {
    pub(crate) fallback_language: LanguageIdentifier,
    pub(crate) languages: Vec<Language>,
}

pub fn use_init_i18n(
    selected_language: LanguageIdentifier,
    fallback_language: LanguageIdentifier,
    languages: impl FnOnce() -> Vec<Language>,
) {
    let selected_language = use_signal(|| selected_language);
    let init_i18_data = use_signal(|| UseInitI18Data {
        languages: languages(),
        fallback_language,
    });

    provide_context(selected_language);
    provide_context(init_i18_data);
}
