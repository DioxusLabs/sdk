use unic_langid::LanguageIdentifier;
use std::collections::HashMap;
use dioxus::prelude::*;
use super::i18n::*;

#[derive(Clone, PartialEq, Copy)]
/// struct for i18n sync (compatible with mutex / rwlock)
pub struct UseI18Sync {
    selected_language: SyncSignal<LanguageIdentifier>,
    data: SyncSignal<I18Data>,
}

impl UseI18Sync {
    /// compares if the language is the selected language
    pub fn is_selected(&self, lang: &LanguageIdentifier) -> bool {
        self.selected_language.read().eq(lang)
    }

    /// use macro ``translate!(i18, "id", param: "value")`` where i18 is ``I18.read().unwrap()``
    pub fn translate_with_params(&self, id: &str, params: HashMap<&str, String>) -> String {
        let mut text = self.translate(id);

        for (name, value) in params {
            text = text.replacen(&format!("{{{name}}}"), &value.to_string(), 1);
        }

        text
    }

    /// use macro ``translate!(i18, "id")`` where i18 is ``I18.read().unwrap()``
    pub fn translate(&self, id: &str) -> String {
        self.data.read().translate(id, &*self.selected_language.read())     
    }

    /// Used to change the app language
    /// 
    /// Example
    /// ```
    /// I18.write().unwrap().set_language("en-US".parse().unwrap());
    /// ```
    pub fn set_language(&mut self, id: LanguageIdentifier) {
        *self.selected_language.write() = id;
    }

    /// Returns a vector of all languages currently stored in the languages map
    /// 
    /// Example for creating buttons with the language name
    /// ```
    /// fn change_language_btn() -> Element {
    ///     let i18_sync = I18.read().unwrap();
    ///     rsx!{{
    ///         i18_sync.language_list().iter().map(|(id, name, _img)| {
    ///             let id = id.clone();
    ///             rsx! { 
    ///                 button {
    ///                     onclick: move |_| { 
    ///                         I18.write().unwrap().set_language(id.clone()); 
    ///                     },
    ///                     "{name}"
    ///        
    ///     }}})}}
    /// }
    /// ```
    pub fn language_list(&self) -> Vec<(LanguageIdentifier, String, String)> {
        self.data.read().language_list()  
    }
}

/// i18n sync initializer, used for multi window applications
/// 
/// Example
/// ```
/// pub(crate) static I18: LazyLock<Arc<RwLock<UseI18Sync>>> = LazyLock::new(|| {
///     Arc::new(RwLock::new(use_i18sync_init(
///         "en-US".parse().unwrap(), // selected
///         ("en-US", HashMap::from([("es", vec!["it-IT"])])).into(), // fallback
///         || {
///             let en_us = Language::from_str(EN_US).unwrap();
///             let es_es = Language::from_str(ES_ES).unwrap();
///             let it_it = Language::from_str(IT_IT).unwrap();
///             vec![en_us, es_es, it_it]
///     })))
/// });
/// ```
pub fn use_i18sync_init(
    selected_language: LanguageIdentifier, 
    fallback_language: FallbackLanguage,
    languages: impl FnOnce() -> Vec<Language>,
) -> UseI18Sync {

    let selected_language = use_signal_sync(|| selected_language);
    let init_i18_data = use_signal_sync(|| I18Data::new(fallback_language, languages));

    UseI18Sync { 
        selected_language: selected_language, 
        data: init_i18_data 
    }
}
