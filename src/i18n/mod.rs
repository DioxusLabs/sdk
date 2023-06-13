use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use unic_langid::LanguageIdentifier;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Language {
    id: LanguageIdentifier,
    texts: Text,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Text {
    Value(String),
    Texts(HashMap<String, Text>),
}

impl Default for Text {
    fn default() -> Self {
        Self::Texts(HashMap::default())
    }
}

impl Text {
    fn query(&self, steps: &mut Vec<&str>) -> Option<String> {
        match self {
            Text::Texts(texts) => {
                if steps.is_empty() {
                    return None;
                }

                let current_path = steps.join(".");

                let this_step = steps.remove(0);
                let deep = texts.get(this_step)?;
                let res = deep.query(steps);
                if res.is_none() {
                    let res_text = texts.get(&current_path);
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

impl FromStr for Language {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(|_| ())
    }
}

impl Language {
    pub fn get_text(&self, path: &str, params: HashMap<&str, String>) -> Option<String> {
        let mut steps = path.split('.').collect::<Vec<&str>>();

        let mut text = self.texts.query(&mut steps).unwrap_or_default();

        for (name, value) in params {
            text = text.replacen(&format!("{{{name}}}"), &value.to_string(), 1);
        }
        Some(text)
    }
}

#[derive(Clone, Copy)]
pub struct UseI18<'a> {
    pub selected_language: &'a UseSharedState<LanguageIdentifier>,
    pub data: &'a UseSharedState<UseInitI18Data>,
}

impl<'a> UseI18<'a> {
    pub fn t_p(&self, id: &str, params: HashMap<&str, String>) -> String {
        let i18n_data = self.data.read();

        // Try searching in the selected language
        for language in i18n_data.languages.iter() {
            if language.id == *self.selected_language.read() {
                return language.get_text(id, params).unwrap_or_default();
            }
        }

        // Otherwise find in the fallback language
        for language in i18n_data.languages.iter() {
            if language.id == i18n_data.fallback_language {
                return language.get_text(id, params).unwrap_or_default();
            }
        }

        // Return the ID as there is no alternative
        id.to_string()
    }

    pub fn t(&self, id: &str) -> String {
        self.t_p(id, HashMap::default())
    }

    pub fn set_language(&self, id: LanguageIdentifier) {
        *self.selected_language.write() = id;
    }
}

pub struct UseInitI18Data {
    fallback_language: LanguageIdentifier,
    languages: Vec<Language>,
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

pub fn use_i18(cx: &ScopeState) -> UseI18 {
    let selected_language = use_shared_state::<LanguageIdentifier>(cx).unwrap();
    let data = use_shared_state::<UseInitI18Data>(cx).unwrap();

    UseI18 {
        selected_language,
        data,
    }
}

#[macro_export]
macro_rules! translate {
    ( $i18:expr, $id:expr, $( $name:ident : $value:expr ),* ) => {
        {
            let mut params_map = HashMap::new();
            $(
                params_map.insert(stringify!($name), $value.to_string());
            )*
            $i18.t_p($id, params_map)
        }
    };

    ( $i18:expr, $id:expr ) => {
        {
            $i18.t_p($id, HashMap::new())
        }
    };
}