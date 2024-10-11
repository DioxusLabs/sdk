use unic_langid::LanguageIdentifier;
use std::{collections::HashMap, str::FromStr};
use serde::{Deserialize, Serialize};

/// Struct for serializing and deserializing the language from json
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Language {
    id: LanguageIdentifier,
    name: Option<String>, // better way to add localized language name
    img: Option<String>,  // better way to connect e.g. flag image with the language
    texts: Text,
}

impl FromStr for Language {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(|_| ())
    }
}

impl Language {
    pub fn get_text(&self, path: &str) -> Option<String> {
        let mut steps = path.split('.').collect::<Vec<&str>>();

        self.texts.query(&mut steps)
    }
}

/// Text value
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum Text {
    Value(String),
    Texts(HashMap<String, Text>),
}

impl Default for Text {
    fn default() -> Self {
        Self::Texts(HashMap::default())
    }
}

impl Text {

    /// queries nested id's that are separated by .
    fn query(&self, steps: &mut Vec<&str>) -> Option<String> {
        match self {
            Text::Texts(value) => {
                if steps.is_empty() {
                    return None;
                }

                let current_path = steps.join(".");

                // Try querying the next step in this list
                let this_step = steps.remove(0);
                let deep = value.get(this_step)?;
                let res = deep.query(steps);

                // If not found try querying by the whole remaining path as if it was the ID
                if res.is_none() {
                    let res_text = value.get(&current_path);
                    if let Some(res_text) = res_text {
                        return res_text.query(steps);
                    }
                }
                res
            }
            Text::Value(value) => Some(value.to_owned()),
        }
    }
}


/// i18n data that stores all fallbacks and languages
pub(crate) struct I18Data {
    fallback_language: FallbackLanguage,
    languages: HashMap<LanguageIdentifier, Language>,
}

impl I18Data {

    /// Creates new i18n struct
    pub(crate) fn new(fallback_language: FallbackLanguage, languages: impl FnOnce() -> Vec<Language>) -> Self {

        let languages_map: HashMap<LanguageIdentifier, Language> = languages()
            .into_iter()
            .map(|language| (language.id.clone(), language))
            .collect();

        I18Data {
            fallback_language,
            languages: languages_map,
        }
    }

    /// Returns a vector of all languages currently stored in the languages map
    pub fn language_list(&self) -> Vec<(LanguageIdentifier, String, String)> {

        // would be better if it could just return iterator
        self.languages
            .values()
            .map(|language| {            
                let name = language.name.clone().unwrap_or_else(|| language.id.to_string());
                let img = language.img.clone().unwrap_or_else(|| "".to_owned());
                (language.id.clone(), name, img)
            }).collect()              
    }

    /// Gets the text from the id, preferably from selected language
    /// if it cannot find it, it looks in fallbacks
    /// otherwise it returns just the id
    pub(crate) fn translate(&self, id: &str, selected_language: &LanguageIdentifier) -> String {

        // Try getting text from the current language
        if let Some(language) = self.languages.get(selected_language) {
            if let Some(text) = language.get_text(id) {
                return text;
            }
        }
        
        // Try getting text from the fallback language
        match &self.fallback_language {
            FallbackLanguage::Single(lang) => {
                if let Some(language) = self.languages.get(lang) {
                    if let Some(text) = language.get_text(id) {
                        return text;
                    }
                }
            },
            FallbackLanguage::Map(fallback) => {
                // Check for matches in the fallback map, can be partial
                if let Some(list) = fallback.map.get(selected_language)
                    .or_else(|| fallback.map.get(&LanguageIdentifier::from_parts(selected_language.language.clone(), None, None, &[]))) {
                    
                    for language in list.iter() {
                        if let Some(language) = self.languages.get(language) {
                            if let Some(text) = language.get_text(id) {
                                return text;
                            }
                        }
                    }
                }
                
                // Fallback to the default language in the map
                if let Some(language) = self.languages.get(&fallback.default) {
                    if let Some(text) = language.get_text(id) {
                        return text;
                    }
                }
            }
        }
        
        // Return a default or empty string if no translation is found
        id.to_string()
    }
}

/// Language fallback 
/// 
/// Map parameters: 
/// ```
/// ("default_fallback_id_full", 
///   HashMap::from([
///     ("language_id_partial_or_full", vec!["preffered_fallback_id_full"])
/// ])).into()
/// ```
/// - partial: will match id even if it's partial. e.g "en" will match "en-US", "en-GB" ...
/// - full: will only match 1:1. e.g "en" will only match "en"
/// 
/// Example
/// ```
/// // Single
/// "en-US".parse().unwrap()
/// 
/// // Map
/// ("en-US", HashMap::from([("es", vec!["it-IT"])])).into()
/// ```
pub enum FallbackLanguage {
    Single(LanguageIdentifier),
    Map(FallbackLanguageMap),
}

/// Fallback language map
/// 
/// Parameters:
/// ```
/// ("default_fallback_id_full", 
///   HashMap::from([
///     ("language_id_partial_or_full", vec!["preffered_fallback_id_full"])
/// ])).into()
/// ```
/// - partial: will match id even if it's partial. e.g "en" will match "en-US", "en-GB" ...
/// - full: will only match 1:1. e.g "en" will only match "en"
pub struct FallbackLanguageMap {
    pub(crate) default: LanguageIdentifier,
    pub(crate) map: HashMap<LanguageIdentifier, Vec<LanguageIdentifier>>,
}

impl FallbackLanguageMap {
    /// creates new fallback map
    pub fn new(default: LanguageIdentifier, map: HashMap<LanguageIdentifier, Vec<LanguageIdentifier>>) -> Self {
        Self { default, map }
    }
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