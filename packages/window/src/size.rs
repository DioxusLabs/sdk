//! Window size utilities.
//!
//! Acces the window size directly in your Dioxus app.
//!
//! #### Platform Support
//! Window size is available on every platform.
//!
//! # Examples
//!
//! ```rust
//! use dioxus::prelude::*;
//! use dioxus_window::size::use_window_size;
//!
//! fn App() -> Element {
//!     let size = use_window_size();
//!     let size = size().unwrap();    
//!
//!     rsx! {
//!         p { "Width: {size.width}" }
//!         p { "Height: {size.height}" }
//!     }
//! }
//! ```
use dioxus::hooks::use_effect;
use dioxus::prelude::{
    ReadOnlySignal, ScopeId, Signal, Writable, provide_root_context, try_use_context, use_hook,
    warnings::signal_write_in_component_body,
};
use dioxus::signals::Readable;
use dioxus::warnings::Warning as _;
use std::error::Error;
use std::fmt::Display;

/// The width and height of a window.
#[derive(Clone, Copy, Debug, Default)]
pub struct WindowSize {
    /// The horizontal size in pixels.
    pub width: u32,
    /// The vertical size in pixels.
    pub height: u32,
}

/// Possible window size errors.
#[derive(Debug, Clone, PartialEq)]
pub enum WindowSizeError {
    /// Window size is not supported on this platform.
    ///
    /// This error only exists for proper SSR hydration.
    Unsupported,
    /// Failed to get the window size.
    CheckFailed,
}

impl Error for WindowSizeError {}
impl Display for WindowSizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => write!(f, "the current platform is not supported"),
            Self::CheckFailed => write!(f, "failed to get the current window size"),
        }
    }
}

type WindowSizeResult = Result<WindowSize, WindowSizeError>;

/// A trait for accessing the inner values of [`WindowSize`].
///
/// These methods can be convenient if you need access to one of the values but not the other.
///
/// # Examples
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_window::size::{use_window_size, ReadableWindowSizeExt};
///
/// fn App() -> Element {
///     let size = use_window_size();
///     
///     let half_of_width = use_memo(move || {
///         let width = size.width().unwrap();
///         width / 2
///     });
///
///     rsx! {
///         div {
///             style: "width: {half_of_width};",
///             "hi"
///         }
///     }
/// }
/// ```
pub trait ReadableWindowSizeExt: Readable<Target = WindowSizeResult> {
    /// Read the width, subscribing to it.
    #[track_caller]
    fn width(&self) -> Result<u32, WindowSizeError> {
        let value = self.read().clone();
        value.map(|x| x.width)
    }

    /// Read the height, subscribing to it.
    #[track_caller]
    fn height(&self) -> Result<u32, WindowSizeError> {
        let value = self.read().clone();
        value.map(|x| x.height)
    }
}

impl<R> ReadableWindowSizeExt for R where R: Readable<Target = WindowSizeResult> {}

/// Get a signal to the window size.
///
/// On first run, the result will be [`WindowSizeError::Unsupported`]. This is to prevent hydration from failing.
/// After the client runs, the window size will be tracked and updated with accurate values.
///
/// # Examples
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_window::size::use_window_size;
///
/// fn App() -> Element {
///     let size = use_window_size();
///     let size = size().unwrap();    
///
///     rsx! {
///         p { "Width: {size.width}" }
///         p { "Height: {size.height}" }
///     }
/// }
/// ```
pub fn use_window_size() -> ReadOnlySignal<WindowSizeResult> {
    let mut window_size = match try_use_context::<Signal<WindowSizeResult>>() {
        Some(w) => w,
        // This should only run once.
        None => {
            let signal = Signal::new_in_scope(Err(WindowSizeError::Unsupported), ScopeId::ROOT);
            provide_root_context(signal)
        }
    };

    // Only start the listener on the client.
    use_effect(move || {
        window_size.set(get_window_size());
        listen(window_size);
    });

    use_hook(|| ReadOnlySignal::new(window_size))
}

// Listener for the web implementation.
#[cfg(target_family = "wasm")]
fn listen(mut window_size: Signal<WindowSizeResult>) {
    use wasm_bindgen::{JsCast, closure::Closure};

    let window = web_sys::window().expect("no wasm window found; are you in wasm?");
    let window2 = window.clone();

    let on_resize = Closure::wrap(Box::new(move || {
        let width = window2
            .inner_width()
            .ok()
            .and_then(|x| x.as_f64())
            .ok_or(WindowSizeError::CheckFailed);

        let height = window2
            .inner_height()
            .ok()
            .and_then(|x| x.as_f64())
            .ok_or(WindowSizeError::CheckFailed);

        let size = (width, height);
        let value = match size {
            (Ok(width), Ok(height)) => Ok(WindowSize {
                width: width as u32,
                height: height as u32,
            }),
            _ => Err(WindowSizeError::CheckFailed),
        };

        signal_write_in_component_body::allow(move || {
            window_size.set(value);
        });
    }) as Box<dyn FnMut()>);

    let on_resize_cb = on_resize.as_ref().clone();
    on_resize.forget();
    window.set_onresize(Some(on_resize_cb.unchecked_ref()));
}

// Listener for anything but the web implementation.
#[cfg(not(target_family = "wasm"))]
fn listen(mut window_size: Signal<WindowSizeResult>) {
    use dioxus_desktop::{WindowEvent, tao::event::Event, window};

    let window = window();
    window.create_wry_event_handler(move |event, _| {
        if let Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } = event
        {
            signal_write_in_component_body::allow(move || {
                window_size.set(Ok(WindowSize {
                    width: size.width,
                    height: size.height,
                }));
            });
        }
    });
}

/// Get the current window size.
///
/// **Note**
///
/// This function will cause hydration to fail if not used inside an effect, task, or event handler.
///
/// # Examples
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_window::size::get_window_size;
///
/// fn App() -> Element {
///     let size = use_signal(get_window_size);
///     let size = size().unwrap();    
///
///     rsx! {
///         p { "Width: {size.width}" }
///         p { "Height: {size.height}" }
///     }
/// }
/// ```
pub fn get_window_size() -> WindowSizeResult {
    get_size_platform()
}

// Web implementation of size getter.
#[cfg(target_family = "wasm")]
fn get_size_platform() -> WindowSizeResult {
    let window = web_sys::window().ok_or(WindowSizeError::CheckFailed)?;

    // We will fail silently for conversion errors.
    let height = window
        .inner_height()
        .ok()
        .and_then(|x| x.as_f64())
        .ok_or(WindowSizeError::CheckFailed)? as u32;

    let width = window
        .inner_width()
        .ok()
        .and_then(|x| x.as_f64())
        .ok_or(WindowSizeError::CheckFailed)? as u32;

    Ok(WindowSize { width, height })
}

// Desktop implementation of size getter.
#[cfg(not(target_family = "wasm"))]
fn get_size_platform() -> WindowSizeResult {
    let window = dioxus_desktop::window();
    let size = window.inner_size();
    Ok(WindowSize {
        width: size.width,
        height: size.height,
    })
}
