use dioxus::prelude::*;
use unic_langid::LanguageIdentifier;

use super::use_i18n::Language;

pub struct UseInitI18Data {
    pub(crate) fallback_language: LanguageIdentifier,
    pub(crate) languages: Vec<Language>,
}

pub fn use_init_i18n(
    cx: &ScopeState,
    selected_language: LanguageIdentifier,
    fallback_language: LanguageIdentifier,
    languages: impl FnOnce() -> Vec<Language>,
) {
    use_shared_state_provider(cx, || selected_language);
    use_shared_state_provider(cx, || UseInitI18Data {
        languages: languages(),
        fallback_language,
    })
}
