//! Provides a clipboard abstraction to access the target system's clipboard.

use copypasta::{ClipboardContext, ClipboardProvider};
use dioxus::prelude::{RefCell, ScopeState};
use std::rc::Rc;

pub fn use_init_clipboard(cx: &ScopeState) {
    cx.use_hook(|| {
        if let Ok(clipboard) = ClipboardContext::new() {
            cx.provide_context(Rc::new(RefCell::new(clipboard)));
        }
    });
}

#[derive(Debug, PartialEq, Clone)]
pub enum ClipboardError {
    FailedToRead,
    FailedToSet,
}

/// Handle to access the ClipboardContent.
#[derive(Clone)]
pub struct UseClipboard {
    clipboard: Rc<RefCell<ClipboardContext>>,
}

impl UseClipboard {
    // Read from the clipboard
    pub fn get(&self) -> Result<String, ClipboardError> {
        self.clipboard
            .borrow_mut()
            .get_contents()
            .map_err(|_| ClipboardError::FailedToRead)
    }

    // Write to the clipboard
    pub fn set(&self, contents: String) -> Result<(), ClipboardError> {
        self.clipboard
            .borrow_mut()
            .set_contents(contents)
            .map_err(|_| ClipboardError::FailedToSet)
    }
}

/// Access the clipboard.
///
/// # Examples
///
/// ```ignore
/// use dioxus_std;
///
/// // Initialize the clipboard
/// use_init_clipboard(cx);
///
/// // Get a handle to the clipboard
/// let clipboard = use_clipboard(cx);
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
pub fn use_clipboard(cx: &ScopeState) -> UseClipboard {
    let clipboard = cx
        .consume_context::<Rc<RefCell<ClipboardContext>>>()
        .expect(
            "Clipboard was not detected. Make sure you initialized it with 'use_init_clipboard'.",
        );
    UseClipboard { clipboard }
}
