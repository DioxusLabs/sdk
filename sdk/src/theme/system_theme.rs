//! Utilities to get and subscribe to the system theme.

use dioxus::prelude::*;
use futures::StreamExt;
use std::{error::Error, fmt::Display, sync::Once};

/// Represents the system theme.
///
/// For any themes other than `light` and `dark`, a [`ColorThemeError::UnknownTheme`] will be returned.
/// We may be able to support custom themes in the future.
#[derive(Debug, Clone, Copy)]
pub enum ColorTheme {
    Light,
    Dark,
}

impl Display for ColorTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Light => write!(f, "light"),
            Self::Dark => write!(f, "dark"),
        }
    }
}

/// Represents an error with system theme utilities.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorThemeError {
    /// System theme is not supported on this platform.
    NotSupported,
    /// Failed to get the system theme.
    CheckFailed,
    /// System returned an unknown theme.
    UnknownTheme,
}

impl Error for ColorThemeError {}
impl Display for ColorThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NotSupported => write!(f, "the current platform is not supported"),
            Self::CheckFailed => write!(
                f,
                "the system returned an error while checking the color theme"
            ),
            Self::UnknownTheme => write!(
                f,
                "the system provided a theme other than `light` or `dark`"
            ),
        }
    }
}

type ResultColorTheme = Result<ColorTheme, ColorThemeError>;

/// A hook for receiving the system theme.
///
/// The initial theme will be returned and updated if the system theme changes.
///
/// # Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_sdk::theme::use_system_theme;
///
/// fn App() -> Element {
///     let theme = use_system_theme();
///
///     rsx! {
///         p {
///             "the current theme is: {theme().unwrap()}"
///         }
///     }
/// }
/// ```
pub fn use_system_theme() -> ReadOnlySignal<ResultColorTheme> {
    let mut system_theme = use_signal(get_system_theme);

    // Initialize the theme listener
    let tx = use_coroutine(|mut rx: UnboundedReceiver<ResultColorTheme>| async move {
        while let Some(data) = rx.next().await {
            system_theme.set(data);
        }
    });

    listen(tx);
    use_hook(|| ReadOnlySignal::new(system_theme))
}

#[allow(dead_code)]
static INIT: Once = Once::new();

/// The listener implementation for wasm targets.
#[cfg(target_family = "wasm")]
fn listen(tx: Coroutine<ResultColorTheme>) {
    use wasm_bindgen::{closure::Closure, JsCast};
    use web_sys::MediaQueryList;

    INIT.call_once(|| {
        let Some(window) = web_sys::window() else {
            tx.send(Err(ColorThemeError::NotSupported));
            return;
        };

        // Get the media query
        let Ok(query) = window.match_media("(prefers-color-scheme: dark)") else {
            tx.send(Err(ColorThemeError::CheckFailed));
            return;
        };

        let Some(query) = query else {
            tx.send(Err(ColorThemeError::UnknownTheme));
            return;
        };

        // Listener that is called when the media query changes.
        // https://developer.mozilla.org/en-US/docs/Web/API/MediaQueryList/change_event
        let listener = Closure::wrap(Box::new(move |query: MediaQueryList| {
            match query.matches() {
                true => tx.send(Ok(ColorTheme::Dark)),
                false => tx.send(Ok(ColorTheme::Light)),
            };
        }) as Box<dyn Fn(MediaQueryList)>);

        let cb = listener.as_ref().clone();
        listener.forget();
        query.set_onchange(Some(cb.unchecked_ref()));
    });
}

/// The listener implementation for desktop targets. (not linux)
#[cfg(not(target_family = "wasm"))]
fn listen(tx: Coroutine<ResultColorTheme>) {
    use dioxus_desktop::{
        tao::{event::Event, window::Theme},
        use_wry_event_handler, WindowEvent,
    };

    use_wry_event_handler(move |event, _| {
        if let Event::WindowEvent {
            event: WindowEvent::ThemeChanged(theme),
            ..
        } = event
        {
            match theme {
                Theme::Dark => tx.send(Ok(ColorTheme::Dark)),
                Theme::Light => tx.send(Ok(ColorTheme::Light)),
                _ => tx.send(Err(ColorThemeError::UnknownTheme)),
            }
        }
    });
}

/// Get the current system theme.
///
/// This function will try to get the current system theme.
///
/// # Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_sdk::theme::{ColorTheme, get_system_theme};
///
/// fn App() -> Element {
///     let theme = use_signal(get_system_theme);
///
///     let class_name = match theme().unwrap() {
///         ColorTheme::Dark => "dark-theme",
///         ColorTheme::Light => "light-theme",
///     };
///
///     rsx! {
///         div {
///             style: "width: 100px; height: 100px;",
///             class: "{class_name}",
///         }
///     }
/// }
/// ```
pub fn get_system_theme() -> ResultColorTheme {
    get_system_theme_platform()
}

/// The wasm implementation to get the system theme.
#[cfg(target_family = "wasm")]
fn get_system_theme_platform() -> ResultColorTheme {
    let Some(window) = web_sys::window() else {
        return Err(ColorThemeError::NotSupported);
    };

    // Check the color theme with a media query
    let Some(query) = window
        .match_media("(prefers-color-scheme: dark)")
        .or(Err(ColorThemeError::CheckFailed))?
    else {
        return Err(ColorThemeError::UnknownTheme);
    };

    match query.matches() {
        true => Ok(ColorTheme::Dark),
        false => Ok(ColorTheme::Light),
    }
}

/// The desktop (except linux) implementation to get the system theme.
#[cfg(not(target_family = "wasm"))]
fn get_system_theme_platform() -> ResultColorTheme {
    use dioxus_desktop::tao::window::Theme;
    use dioxus_desktop::DesktopContext;

    // Get window context and theme
    let Some(window) = try_consume_context::<DesktopContext>() else {
        return Err(ColorThemeError::NotSupported);
    };
    let theme = window.theme();

    match theme {
        Theme::Light => Ok(ColorTheme::Light),
        Theme::Dark => Ok(ColorTheme::Dark),
        _ => Err(ColorThemeError::UnknownTheme),
    }
}

// //! Provides a hook to access the user's preferred color scheme.
// use dioxus::prelude::*;
// use std::{fmt, sync::Once};
// use wasm_bindgen::{prelude::Closure, JsCast};

// /// Identifies the user's preferred color scheme.
// #[derive(Debug, Clone)]
// pub enum PreferredColorScheme {
//     Light,
//     Dark,
// }

// static INIT: Once = Once::new();

// /// Retrieves (as well as listens for changes) to the user's preferred color scheme (dark or light) so your application can adapt accordingly.
// pub fn use_preferred_color_scheme() -> Result<PreferredColorScheme, PreferredColorSchemeError> {
//     // This code is kinda messy..
//     let window = match web_sys::window() {
//         Some(w) => w,
//         None => {
//             return Err(PreferredColorSchemeError::NotSupported(
//                 "this feature is only supported on web assembly".to_string(),
//             ))
//         }
//     };

//     let media_query_list = match window.match_media("(prefers-color-scheme: dark)") {
//         Ok(opt) => match opt {
//             Some(m) => m,
//             None => {
//                 return Err(PreferredColorSchemeError::SchemeCheckFailed(
//                     "failed to determine color scheme".to_string(),
//                 ))
//             }
//         },
//         Err(e) => {
//             return Err(PreferredColorSchemeError::SchemeCheckFailed(
//                 e.as_string().unwrap_or(
//                     "Failed to determine preferred scheme and couldn't retrieve error".to_string(),
//                 ),
//             ))
//         }
//     };

//     let update_callback = schedule_update();

//     // Create closure that listens to the event of matchMedia and calls write to scheme
//     INIT.call_once(|| {
//         let listener = Closure::wrap(Box::new(move || {
//             update_callback();
//         }) as Box<dyn Fn()>);

//         // Create a reference to the closure to pass to JavaScript.
//         let cb = listener.as_ref().clone();

//         // Prevent the closure from being dropped.
//         // This normally isn't good practice, however the idea is that this callback should live forever.
//         listener.forget();

//         media_query_list.set_onchange(Some(cb.unchecked_ref()));
//     });

//     Ok(determine_scheme(media_query_list.matches()))
// }

// fn determine_scheme(value: bool) -> PreferredColorScheme {
//     match value {
//         true => PreferredColorScheme::Dark,
//         false => PreferredColorScheme::Light,
//     }
// }

// /// Represents errors when utilizing the preferred color scheme hook.
// #[derive(Debug)]
// pub enum PreferredColorSchemeError {
//     /// Returned when used on an unsupported platform.
//     NotSupported(String),
//     /// Represents a failure when determining the user's preferred color scheme.
//     SchemeCheckFailed(String),
// }

// impl std::error::Error for PreferredColorSchemeError {}
// impl fmt::Display for PreferredColorSchemeError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             PreferredColorSchemeError::SchemeCheckFailed(s) => write!(f, "{}", s),
//             PreferredColorSchemeError::NotSupported(s) => write!(f, "{}", s),
//         }
//     }
// }
