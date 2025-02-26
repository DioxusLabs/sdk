//! Window size utilities.

use dioxus::hooks::use_effect;
use dioxus::prelude::{
    provide_root_context, try_use_context, use_hook, warnings::signal_write_in_component_body,
    ReadOnlySignal, ScopeId, Signal, Writable,
};
use dioxus::warnings::Warning as _;

/// The width and height of a window.
#[derive(Clone, Copy, Debug, Default)]
pub struct WindowSize {
    /// The horizontal size in pixels.
    pub width: u32,
    /// The vertical size in pixels.
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WindowSizeError {
    /// Window size is not supported on this platform.
    Unsupported,
    /// Failed to get the window size.
    CheckFailed,
}

type WindowSizeResult = Result<WindowSize, WindowSizeError>;

/// A hook for subscribing to the window size.
///
/// The initial window size will be returned with this hook and updated continously as the window is resized.
///
/// # Examples
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_window::size::use_window_size;
///
/// fn App() -> Element {
///     let size = use_window_size();
///
///     rsx! {
///         p { "Width: {size().width}" }
///         p { "Height: {size().height}" }
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
    use std::sync::Once;
    use wasm_bindgen::{closure::Closure, JsCast, JsValue};

    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let window = web_sys::window().expect("no wasm window found; are you in wasm?");
        let window2 = window.clone();

        // Todo: Actually handle errors here.
        let on_resize = Closure::wrap(Box::new(move || {
            let height = window2
                .inner_height()
                .unwrap_or(JsValue::from_f64(0.0))
                .as_f64()
                .unwrap_or(0.0) as u32;

            let width = window2
                .inner_width()
                .unwrap_or(JsValue::from_f64(0.0))
                .as_f64()
                .unwrap_or(0.0) as u32;

            signal_write_in_component_body::allow(move || {
                window_size.set(Ok(WindowSize { width, height }));
            });
        }) as Box<dyn FnMut()>);

        let on_resize_cb = on_resize.as_ref().clone();
        on_resize.forget();
        window.set_onresize(Some(on_resize_cb.unchecked_ref()));
    });
}

// Listener for anything but the web implementation.
#[cfg(not(target_family = "wasm"))]
fn listen(mut window_size: Signal<WindowSizeResult>) {
    use dioxus_desktop::{tao::event::Event, window, WindowEvent};

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
///
///     rsx! {
///         p { "Width: {size().width}" }
///         p { "Height: {size().height}" }
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
