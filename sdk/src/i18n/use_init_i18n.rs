use dioxus::prelude::*;
use unic_langid::LanguageIdentifier;
use std::collections::HashMap;

use super::use_i18n::Language;

pub struct UseInitI18Data {
    pub(crate) fallback_language: FallbackLanguage,
    pub(crate) languages: HashMap<LanguageIdentifier, Language>,
}

pub(crate) struct FallbackLanguageMap {
    pub(crate) default: LanguageIdentifier,
    pub(crate) map: HashMap<LanguageIdentifier, Vec<LanguageIdentifier>>,
}

impl FallbackLanguageMap {
    pub(crate) fn new(default: LanguageIdentifier, map: HashMap<LanguageIdentifier, Vec<LanguageIdentifier>>) -> Self {
        Self { default, map }
    }
}

pub enum FallbackLanguage {
    Single(LanguageIdentifier),
    Map(FallbackLanguageMap),
}

impl From<&str> for FallbackLanguage {
    fn from(value: &str) -> Self {
        Self::Single(value.parse().unwrap())
    }
}

impl From<LanguageIdentifier> for FallbackLanguage {
    fn from(value: LanguageIdentifier) -> Self {
        Self::Single(value)
    }
}

impl From<(LanguageIdentifier, HashMap<LanguageIdentifier, Vec<LanguageIdentifier>>)> for FallbackLanguage {
    fn from(value: (LanguageIdentifier, HashMap<LanguageIdentifier, Vec<LanguageIdentifier>>)) -> Self {
        let (default, map) = value;
        Self::Map(FallbackLanguageMap::new(default, map))
    }
}

impl From<(&str, HashMap<&str, Vec<&str>>)> for FallbackLanguage {
    fn from(value: (&str, HashMap<&str, Vec<&str>>)) -> Self {
        let (default_str, map_str) = value;
        let default: LanguageIdentifier = default_str.parse().unwrap();
        
        let map: HashMap<LanguageIdentifier, Vec<LanguageIdentifier>> = map_str.into_iter()
            .map(|(k, v)| {
                let key: LanguageIdentifier = k.parse().unwrap();
                let values: Vec<LanguageIdentifier> = v.into_iter().map(|s| s.parse().unwrap()).collect();
                (key, values)
            })
            .collect();

        Self::Map(FallbackLanguageMap::new(default, map))
    }
}

pub fn use_init_i18n(
    selected_language: LanguageIdentifier,
    fallback_language: FallbackLanguage,
    languages: impl FnOnce() -> Vec<Language>,
) {
    let selected_language = use_signal(|| selected_language);

    let languages_vec = languages();
    let languages_map = languages_vec.into_iter()
        .map(|language| (language.id.clone(), language))  // Clone or copy id as needed
        .collect::<HashMap<_, _>>();

    let init_i18_data = use_signal(|| UseInitI18Data {
        languages: languages_map,
        fallback_language: fallback_language,
    });

    provide_context(selected_language);
    provide_context(init_i18_data);
}
