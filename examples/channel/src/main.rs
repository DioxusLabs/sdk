use dioxus::{logger::tracing::info, prelude::*};
use dioxus_sync::channel::{use_channel, use_listen_channel};

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    let channel = use_channel::<String>(5);

    use_listen_channel(&channel, |message| async {
        match message {
            Ok(value) => info!("Incoming message: {value}"),
            Err(err) => info!("Error: {err:?}"),
        }
    });

    let send = move |_: MouseEvent| {
        to_owned![channel];
        async move {
            channel.send("Hello").await.ok();
        }
    };

    rsx!(
        button {
            onclick: send,
            "Send hello"
        }
    )
}
