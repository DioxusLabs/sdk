//! Utilities to get and subscribe to the system theme.

use dioxus::prelude::*;
use std::{error::Error, fmt::Display};

/// Represents the system theme.
///
/// For any themes other than `light` and `dark`, a [`ColorThemeError::UnknownTheme`] will be returned.
/// We may be able to support custom themes in the future.
#[derive(Debug, Clone, Copy)]
pub enum SystemTheme {
    Light,
    Dark,
}

impl Display for SystemTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Light => write!(f, "light"),
            Self::Dark => write!(f, "dark"),
        }
    }
}

/// Represents an error with system theme utilities.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemThemeError {
    /// System theme is not supported on this platform.
    NotSupported,
    /// Failed to get the system theme.
    CheckFailed,
    /// System returned an unknown theme.
    UnknownTheme,
}

impl Error for SystemThemeError {}
impl Display for SystemThemeError {
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

type SystemThemeResult = Result<SystemTheme, SystemThemeError>;

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
pub fn use_system_theme() -> ReadOnlySignal<SystemThemeResult> {
    let system_theme = match try_use_context::<Signal<SystemThemeResult>>() {
        Some(s) => s,
        // This should only run once.
        None => {
            let theme = provide_root_context(Signal::new(get_system_theme()));
            listen(theme);
            theme
        }
    };

    use_hook(|| ReadOnlySignal::new(system_theme))
}

/// The listener implementation for wasm targets.
/// This should only be called once.
#[cfg(target_family = "wasm")]
fn listen(mut theme: Signal<SystemThemeResult>) {
    use wasm_bindgen::{closure::Closure, JsCast};
    use web_sys::MediaQueryList;

    let Some(window) = web_sys::window() else {
        theme.set(Err(SystemThemeError::NotSupported));
        return;
    };

    // Get the media query
    let Ok(query) = window.match_media("(prefers-color-scheme: dark)") else {
        theme.set(Err(SystemThemeError::CheckFailed));
        return;
    };

    let Some(query) = query else {
        theme.set(Err(SystemThemeError::UnknownTheme));
        return;
    };

    // Listener that is called when the media query changes.
    // https://developer.mozilla.org/en-US/docs/Web/API/MediaQueryList/change_event
    let listener = Closure::wrap(Box::new(move |query: MediaQueryList| {
        match query.matches() {
            true => theme.set(Ok(SystemTheme::Dark)),
            false => theme.set(Ok(SystemTheme::Light)),
        };
    }) as Box<dyn FnMut(MediaQueryList)>);

    let cb = listener.as_ref().clone();
    listener.forget();
    query.set_onchange(Some(cb.unchecked_ref()));
}

/// The listener implementation for desktop targets. (not linux)
/// This should only be called once.
#[cfg(not(target_family = "wasm"))]
fn listen(mut theme: Signal<SystemThemeResult>) {
    use dioxus_desktop::{
        tao::{event::Event, window::Theme},
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
                Theme::Dark => theme.set(Ok(SystemTheme::Dark)),
                Theme::Light => theme.set(Ok(SystemTheme::Light)),
                _ => theme.set(Err(SystemThemeError::UnknownTheme)),
            };
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
/// use dioxus_sdk::theme::{SystemTheme, get_system_theme};
///
/// fn App() -> Element {
///     let theme = use_signal(get_system_theme);
///
///     let class_name = match theme().unwrap() {
///         SystemTheme::Dark => "dark-theme",
///         SystemTheme::Light => "light-theme",
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
pub fn get_system_theme() -> SystemThemeResult {
    get_system_theme_platform()
}

/// The wasm implementation to get the system theme.
#[cfg(target_family = "wasm")]
fn get_system_theme_platform() -> SystemThemeResult {
    let Some(window) = web_sys::window() else {
        return Err(SystemThemeError::NotSupported);
    };

    // Check the color theme with a media query
    let Some(query) = window
        .match_media("(prefers-color-scheme: dark)")
        .or(Err(SystemThemeError::CheckFailed))?
    else {
        return Err(SystemThemeError::UnknownTheme);
    };

    match query.matches() {
        true => Ok(SystemTheme::Dark),
        false => Ok(SystemTheme::Light),
    }
}

/// The desktop (except linux) implementation to get the system theme.
#[cfg(not(target_family = "wasm"))]
fn get_system_theme_platform() -> SystemThemeResult {
    use dioxus_desktop::tao::window::Theme;
    use dioxus_desktop::DesktopContext;

    // Get window context and theme
    let Some(window) = try_consume_context::<DesktopContext>() else {
        return Err(SystemThemeError::NotSupported);
    };
    let theme = window.theme();

    match theme {
        Theme::Light => Ok(SystemTheme::Light),
        Theme::Dark => Ok(SystemTheme::Dark),
        _ => Err(SystemThemeError::UnknownTheme),
    }
}
