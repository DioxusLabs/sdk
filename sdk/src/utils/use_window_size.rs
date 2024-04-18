use dioxus::prelude::*;
use futures_util::stream::StreamExt;
use std::sync::Once;

#[allow(dead_code)]
static INIT: Once = Once::new();
type WindowSize = (u32, u32);

pub fn use_window_size() -> WindowSize {
    let mut window_size = use_signal(get_window_size);

    // Initialize the handler
    let tx = use_coroutine(|mut rx: UnboundedReceiver<WindowSize>| async move {
        while let Some(data) = rx.next().await {
            window_size.set(data);
        }
    });

    listen(tx);

    *window_size.read_unchecked()
}

// Listener for the web implementation.
#[cfg(target_arch = "wasm32")]
fn listen(tx: Coroutine<WindowSize>) {
    use wasm_bindgen::{closure::Closure, JsCast, JsValue};

    INIT.call_once(|| {
        let window = web_sys::window().expect("no wasm window found; are you in wasm?");
        let window2 = window.clone();

        // We will fail silently for conversion errors.
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

            tx.send((width, height));
        }) as Box<dyn FnMut()>);

        let on_resize_cb = on_resize.as_ref().clone();
        on_resize.forget();
        window.set_onresize(Some(on_resize_cb.unchecked_ref()));
    });
}

// Web implementation of size getter.
#[cfg(target_arch = "wasm32")]
pub fn get_window_size() -> WindowSize {
    use wasm_bindgen::JsValue;
    let window = web_sys::window().expect("no wasm window found; are you in wasm?");

    // We will fail silently for conversion errors.
    let height = window
        .inner_height()
        .unwrap_or(JsValue::from_f64(0.0))
        .as_f64()
        .unwrap_or(0.0) as u32;

    let width = window
        .inner_width()
        .unwrap_or(JsValue::from_f64(0.0))
        .as_f64()
        .unwrap_or(0.0) as u32;

    (width, height)
}

// Listener for anything but the web implementation.
#[cfg(not(target_arch = "wasm32"))]
fn listen(tx: Coroutine<WindowSize>) {
    use dioxus_desktop::{tao::event::Event, use_wry_event_handler, WindowEvent};

    use_wry_event_handler(move |event, _| {
        if let Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } = event
        {
            tx.send((size.width, size.height));
        }
    });
}

// Desktop implementation of size getter.
#[cfg(not(target_arch = "wasm32"))]
pub fn get_window_size() -> WindowSize {
    let window = dioxus_desktop::window();
    let size = window.inner_size();
    (size.width, size.height)
}
