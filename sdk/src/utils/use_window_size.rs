use std::sync::Once;

use dioxus::prelude::*;
use futures_util::stream::StreamExt;
use wasm_bindgen::{closure::Closure, JsCast};

static INIT: Once = Once::new();

type WindowSize = (f64, f64);

pub fn use_window_size() -> WindowSize {
    let mut window_size = use_signal(|| (0.0, 0.0));

    // Initialize the handler
    let tx = use_coroutine(|mut rx: UnboundedReceiver<WindowSize>| async move {
        while let Some(data) = rx.next().await {
            window_size.set(data);
        }
    });

    // Start the listener
    INIT.call_once(|| {
        listen(tx);
    });

    window_size.read_unchecked().clone()
}

// Listener for the web implementation.
#[cfg(target_arch = "wasm32")]
fn listen(tx: Coroutine<WindowSize>) {
    let window = web_sys::window().expect("no window, are you in wasm?");
    let window2 = window.clone();

    let on_resize = Closure::wrap(Box::new(move || {
        let height = window2.inner_height().unwrap().as_f64().unwrap();
        let width = window2.inner_width().unwrap().as_f64().unwrap();

        tx.send((width, height));
    }) as Box<dyn FnMut()>);

    let on_resize_cb = on_resize.as_ref().clone();
    on_resize.forget();
    window.set_onresize(Some(on_resize_cb.unchecked_ref()));
}

// Listener for anything but the web implementation.
#[cfg(not(target_arch = "wasm32"))]
fn listen(tx: Coroutine<WindowSize>) {
    todo!()
}
