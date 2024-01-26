//! Provides a clipboard abstraction to access the target system's clipboard.

use copypasta::{ClipboardContext, ClipboardProvider};
use dioxus::prelude::*;

#[derive(Debug, PartialEq, Clone)]
pub enum ClipboardError {
    FailedToRead,
    FailedToSet,
}

/// Handle to access the ClipboardContext.
#[derive(Clone)]
pub struct UseClipboard {
    clipboard: Signal<ClipboardContext>,
}

impl UseClipboard {
    // Read from the clipboard
    pub fn get(&self) -> Result<String, ClipboardError> {
        self.clipboard
            .write()
            .get_contents()
            .map_err(|_| ClipboardError::FailedToRead)
    }

    // Write to the clipboard
    pub fn set(&self, contents: String) -> Result<(), ClipboardError> {
        self.clipboard
            .write()
            .set_contents(contents)
            .map_err(|_| ClipboardError::FailedToSet)
    }
}

/// Access the clipboard.
///
/// # Examples
///
/// ```ignore
/// use dioxus_std::clipboard::use_clipboard;
///
/// // Get a handle to the clipboard
/// let clipboard = use_clipboard();
///
/// // Read the clipboard content
/// if let Ok(content) = clipboard.get() {
///     println!("{}", content);
/// }
///
/// // Write to the clipboard
/// clipboard.set("Hello, Dioxus!".to_string());;
///  
/// ```
pub fn use_clipboard() -> UseClipboard {
    let clipboard = match consume_context() {
        Some(rt) => rt,
        None => {
            let clipboard = ClipboardContext::new().expect("Cannot create Clipboard.");
            provide_root_context(Signal::new(clipboard)).unwrap()
        }
    };
    UseClipboard { clipboard }
}
