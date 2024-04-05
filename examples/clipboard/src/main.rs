use dioxus::prelude::*;
use dioxus_sdk::clipboard::use_clipboard;

fn main() {
    dioxus_desktop::launch(app);
}

fn app() -> Element {
    let mut clipboard = use_clipboard();
    let mut text = use_signal(String::new);

    let oninput = move |e: FormEvent| {
        text.set(e.data.value.clone());
    };

    let oncopy = move |_| match clipboard.set(text.read().clone()) {
        Ok(_) => println!("Copied to clipboard: {}", text.read()),
        Err(err) => println!("Error on copy: {err:?}"),
    };

    let onpaste = move |_| match clipboard.get() {
        Ok(contents) => {
            println!("Pasted from clipboard: {contents}");
            text.set(contents);
        }
        Err(err) => println!("Error on paste: {err:?}"),
    };

    rsx!(
        input {
            oninput: oninput,
            value: "{text}"
        }
        button {
            onclick: oncopy,
            "Copy"
        }
        button {
            onclick: onpaste,
            "Paste"
        }
    )
}
