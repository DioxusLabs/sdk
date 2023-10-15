use dioxus::prelude::*;
use dioxus_std::clipboard::{use_clipboard, use_init_clipboard};

fn main() {
    dioxus_desktop::launch(app);
}

fn app(cx: Scope) -> Element {
    use_init_clipboard(cx);
    let clipboard = use_clipboard(cx);
    let text = use_state(cx, String::new);

    let oninput = |e: FormEvent| {
        text.set(e.data.value.clone());
    };

    let oncopy = {
        to_owned![clipboard];
        move |_| match clipboard.set(text.get().clone()) {
            Ok(_) => println!("Copied to clipboard: {}", text.get()),
            Err(err) => println!("Error on copy: {err:?}"),
        }
    };

    let onpaste = move |_| match clipboard.get() {
        Ok(contents) => {
            println!("Pasted from clipboard: {contents}");
            text.set(contents);
        }
        Err(err) => println!("Error on paste: {err:?}"),
    };

    render!(
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
