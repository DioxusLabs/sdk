use dioxus::logger::tracing::{info, Level};
use dioxus::prelude::*;
use dioxus_util::select_file::{
    select_file, select_file_base64, select_file_text, select_files, select_files_base64,
    select_files_text, FilePickerOptions,
};

fn main() {
    dioxus::logger::init(Level::TRACE).unwrap();
    launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 0.5rem;",
            h1 { "File Picker Examples" }

            button {
                onclick: move |_| async move {
                    let file = select_file_base64(&FilePickerOptions::default())
                        .await
                        .unwrap();
                    if let Some(file) = file {
                        info!("Selected a file with base64 data: {:?}", file);
                    } else {
                        info!("No file selected");
                    }
                },
                "Select one file with base64 data"
            }

            button {
                onclick: move |_| async move {
                    let files = select_files_base64(&FilePickerOptions::default())
                        .await
                        .unwrap();
                    if files.is_empty() {
                        info!("No files selected");
                    } else {
                        for file in files {
                            info!("Selected file with base64 data: {:?}", file);
                        }
                    }
                },
                "Select multiple files with base64 data"
            }

            button {
                onclick: move |_| async move {
                    let file = select_file_text(&FilePickerOptions::default())
                        .await
                        .unwrap();
                    if let Some(file) = file {
                        info!("Selected a file with text data: {:?}", file);
                    } else {
                        info!("No file selected");
                    }
                },
                "Select one file with text data"
            }

            button {
                onclick: move |_| async move {
                    let files = select_files_text(&FilePickerOptions::default())
                        .await
                        .unwrap();
                    if files.is_empty() {
                        info!("No files selected");
                    } else {
                        for file in files {
                            info!("Selected file with text data: {:?}", file);
                        }
                    }
                },
                "Select multiple files with text data"
            }

            button {
                onclick: move |_| async move {
                    let file = select_file(&FilePickerOptions::default()).await.unwrap();
                    if let Some(file) = file {
                        info!("Selected a file with metadata only: {:?}", file);
                    } else {
                        info!("No file selected");
                    }
                },
                "Select one file (metadata only)"
            }

            button {
                onclick: move |_| async move {
                    let files = select_files(&FilePickerOptions::default()).await.unwrap();
                    if files.is_empty() {
                        info!("No files selected");
                    } else {
                        for file in files {
                            info!("Selected file with metadata only: {:?}", file);
                        }
                    }
                },
                "Select multiple files (metadata only)"
            }
        }
    }
}
