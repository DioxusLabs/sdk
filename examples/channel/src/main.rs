use dioxus::prelude::*;
use dioxus_std::channel::{use_channel, use_listen_channel};

fn main() {
    // init debug tool for WebAssembly
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();

    dioxus_web::launch(app);
}

fn app(cx: Scope) -> Element {
    let channel = use_channel::<String>(cx, 5);

    use_listen_channel(cx, &channel, move |message| async move {
        match message {
            Ok(value) => log::info!("Incoming message: {value}"),
            Err(err) => log::info!("Error: {err:?}"),
        }
    });

    let send = move |_: MouseEvent| {
        to_owned![channel];
        async move {
            channel.send("Hello").await.ok();
        }
    };

    render!(
        button {
            onclick: send,
            "Send hello"
        }
    )
}
