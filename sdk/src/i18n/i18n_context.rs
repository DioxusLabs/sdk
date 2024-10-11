use unic_langid::LanguageIdentifier;
use std::collections::HashMap;
use dioxus::prelude::*;
use super::i18n::*;

/// i18n unsync initializer, used for web and single window applications
/// 
/// Example
/// ```
/// fn app() -> Element {
///     use_init_i18n(
///         "en-US".parse().unwrap(), // selected
///         ("en-US", HashMap::from([("es", vec!["it-IT"])])).into(), // fallback
///         || {
///             let en_us = Language::from_str(EN_US).unwrap();
///             let es_es = Language::from_str(ES_ES).unwrap();
///             let it_it = Language::from_str(IT_IT).unwrap();
///             vec![en_us, es_es, it_it]
///     });
/// 
///     let i18 = use_i18();
/// 
///     rsx!({translate!(i18, "messages.hello", name: "Dioxus")})
/// }
/// ```
pub fn use_init_i18n(
    selected_language: LanguageIdentifier,
    fallback_language: FallbackLanguage,
    languages: impl FnOnce() -> Vec<Language>,
) {
    let selected_language = use_signal(|| selected_language);
    let init_i18_data = use_signal(|| I18Data::new(fallback_language, languages));

    provide_context(selected_language);
    provide_context(init_i18_data);
}

/// struct for i18n unsync
#[derive(Clone, PartialEq, Copy)]
pub struct UseI18 {
    selected_language: Signal<LanguageIdentifier>,
    data: Signal<I18Data>,
}

impl UseI18 {
    /// compares if the language is the selected language
    pub fn is_selected(&self, lang: &LanguageIdentifier) -> bool {
        self.selected_language.read().eq(lang)
    }

    /// use macro ``translate!(i18, "id", param: "value")``
    pub fn translate_with_params(&self, id: &str, params: HashMap<&str, String>) -> String {
        let mut text = self.translate(id);

        for (name, value) in params {
            text = text.replacen(&format!("{{{name}}}"), &value.to_string(), 1);
        }

        text
    }

    /// use macro ``translate!(i18, "id")``
    pub fn translate(&self, id: &str) -> String {
        self.data.read().translate(id, &*self.selected_language.read())     
    }

    /// Used to change the app language
    /// 
    /// Example
    /// ```
    /// let mut i18 = use_i18();
    /// i18.set_language("en-US".parse().unwrap());
    /// ```
    pub fn set_language(&mut self, id: LanguageIdentifier) {
        *self.selected_language.write() = id;
    }

    /// Returns a vector of all languages currently stored in the languages map
    /// 
    /// Example for creating buttons with the language name
    /// ```
    /// fn change_language_btn() -> Element {
    ///     let mut i18 = use_i18();
    ///     rsx!{{
    ///         i18.language_list().iter().map(|(id, name, _img)| {
    ///             let id = id.clone();
    ///             rsx! { 
    ///                 button {
    ///                     onclick: move |_| { i18.set_language(id.clone()); },
    ///                     "{name}"
    ///        
    ///     }}})}}
    /// }
    /// ```
    pub fn language_list(&self) -> Vec<(LanguageIdentifier, String, String)> {
        self.data.read().language_list()  
    }
}

/// use i18n unsyn
/// 
/// Example
/// ```
/// fn app_part() -> Element {
///     let i18 = use_i18();
/// 
///     rsx!({translate!(i18, "messages.hello", name: "Dioxus")})
/// }
/// ```
pub fn use_i18() -> UseI18 {
    use_hook(|| {
        let selected_language = consume_context::<Signal<LanguageIdentifier>>();
        let data = consume_context::<Signal<I18Data>>();

        UseI18 {
            selected_language,
            data,
        }
    })
}
