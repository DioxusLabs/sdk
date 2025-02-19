//! Window Theme

use dioxus::prelude::*;
use std::{error::Error, fmt::Display};

/// Represents the system theme.
///
/// For any themes other than `light` and `dark`, a [`ThemeError::UnknownTheme`] will be returned.
/// We may be able to support custom themes in the future.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    Light,
    Dark,
}

impl Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Light => write!(f, "light"),
            Self::Dark => write!(f, "dark"),
        }
    }
}

/// Represents an error with system theme utilities.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeError {
    /// Theme is not supported on this platform.
    NotSupported,
    /// Failed to get the system theme.
    CheckFailed,
    /// System returned an unknown theme.
    UnknownTheme,
}

impl Error for ThemeError {}
impl Display for ThemeError {
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

type ThemeResult = Result<Theme, ThemeError>;

/// A hook for receiving the system theme.
///
/// The initial theme will be returned and updated if the system theme changes.
///
/// # Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_window::theme::use_theme;
///
/// fn App() -> Element {
///     let theme = use_theme();
///
///     rsx! {
///         p {
///             "the current theme is: {theme().unwrap()}"
///         }
///     }
/// }
/// ```
pub fn use_theme() -> ReadOnlySignal<ThemeResult> {
    let system_theme = match try_use_context::<Signal<ThemeResult>>() {
        Some(s) => s,
        // This should only run once.
        None => {
            let signal = Signal::new_in_scope(get_theme(), ScopeId::ROOT);
            let theme = provide_root_context(signal);
            listen(theme);
            theme
        }
    };

    use_hook(|| ReadOnlySignal::new(system_theme))
}

/// The listener implementation for wasm targets.
/// This should only be called once.
#[cfg(target_family = "wasm")]
fn listen(mut theme: Signal<ThemeResult>) {
    use wasm_bindgen::{closure::Closure, JsCast};
    use web_sys::MediaQueryList;

    let Some(window) = web_sys::window() else {
        theme.set(Err(ThemeError::NotSupported));
        return;
    };

    // Get the media query
    let Ok(query) = window.match_media("(prefers-color-scheme: dark)") else {
        theme.set(Err(ThemeError::CheckFailed));
        return;
    };

    let Some(query) = query else {
        theme.set(Err(ThemeError::UnknownTheme));
        return;
    };

    // Listener that is called when the media query changes.
    // https://developer.mozilla.org/en-US/docs/Web/API/MediaQueryList/change_event
    let listener = Closure::wrap(Box::new(move |query: MediaQueryList| {
        match query.matches() {
            true => theme.set(Ok(Theme::Dark)),
            false => theme.set(Ok(Theme::Light)),
        };
    }) as Box<dyn FnMut(MediaQueryList)>);

    let cb = listener.as_ref().clone();
    listener.forget();
    query.set_onchange(Some(cb.unchecked_ref()));
}

/// The listener implementation for desktop targets. (not linux)
/// This should only be called once.
#[cfg(not(target_family = "wasm"))]
fn listen(mut theme: Signal<ThemeResult>) {
    use dioxus_desktop::{
        tao::{event::Event, window::Theme as TaoTheme},
        window, WindowEvent,
    };

    let window = window();

    window.create_wry_event_handler(move |event, _| {
        if let Event::WindowEvent {
            event: WindowEvent::ThemeChanged(new_theme),
            ..
        } = event
        {
            match new_theme {
                TaoTheme::Dark => theme.set(Ok(Theme::Dark)),
                TaoTheme::Light => theme.set(Ok(Theme::Light)),
                _ => theme.set(Err(ThemeError::UnknownTheme)),
            };
        }
    });
}

/// Get the current theme.
///
/// This function will try to get the current theme.
///
/// # Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_window::theme::{Theme, get_theme};
///
/// fn App() -> Element {
///     let theme = use_signal(get_theme);
///
///     let class_name = match theme().unwrap() {
///         Theme::Dark => "dark-theme",
///         Theme::Light => "light-theme",
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
pub fn get_theme() -> ThemeResult {
    get_theme_platform()
}

/// The wasm implementation to get the system theme.
#[cfg(target_family = "wasm")]
fn get_theme_platform() -> ThemeResult {
    let Some(window) = web_sys::window() else {
        return Err(ThemeError::NotSupported);
    };

    // Check the color theme with a media query
    let Some(query) = window
        .match_media("(prefers-color-scheme: dark)")
        .or(Err(ThemeError::CheckFailed))?
    else {
        return Err(ThemeError::UnknownTheme);
    };

    match query.matches() {
        true => Ok(Theme::Dark),
        false => Ok(Theme::Light),
    }
}

/// The desktop (except linux) implementation to get the system theme.
#[cfg(not(target_family = "wasm"))]
fn get_theme_platform() -> ThemeResult {
    use dioxus_desktop::tao::window::Theme as TaoTheme;
    use dioxus_desktop::DesktopContext;

    // Get window context and theme
    let Some(window) = try_consume_context::<DesktopContext>() else {
        return Err(ThemeError::NotSupported);
    };
    let theme = window.theme();

    match theme {
        TaoTheme::Light => Ok(Theme::Light),
        TaoTheme::Dark => Ok(Theme::Dark),
        _ => Err(ThemeError::UnknownTheme),
    }
}
