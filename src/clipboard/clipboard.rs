//! Provides a clipboard abstraction to access the target system's clipboard.

use copypasta::{ClipboardContext, ClipboardProvider};
use std::fmt;

/// Contains the context for interacting with the clipboard.
///
/// # Examples
///
/// ```
/// use dioxus_std;
///
/// // Access the clipboard abstraction
/// let mut clipboard = dioxus_std::clipboard::Clipboard::new().unwrap();
///
/// // Get clipboard content
/// let contents = clipboard.get_contents().unwrap();
/// println!("{}", contents);
///
/// // Set clipboard content
/// clipboard.set_content("Hello, Dioxus!".to_string()).unwrap();
///  
/// ```
pub struct Clipboard {
    ctx: ClipboardContext,
}

impl Clipboard {
    /// Creates a new struct to utilize the clipboard abstraction.
    pub fn new() -> Result<Self, ClipboardError> {
        let ctx = match ClipboardContext::new() {
            Ok(ctx) => ctx,
            Err(e) => return Err(ClipboardError::FailedToInit(e.to_string())),
        };

        Ok(Self { ctx })
    }

    /// Provides a [`String`] of the target system's current clipboard content.
    pub fn get_content(&mut self) -> Result<String, ClipboardError> {
        match self.ctx.get_contents() {
            Ok(content) => Ok(content),
            Err(e) => return Err(ClipboardError::FailedToFetchContent(e.to_string())),
        }
    }

    /// Set the clipboard's content to the provided [`String`]
    pub fn set_content(&mut self, value: String) -> Result<(), ClipboardError> {
        match self.ctx.set_contents(value) {
            Ok(()) => Ok(()),
            Err(e) => Err(ClipboardError::FailedToSetContent(e.to_string())),
        }
    }
}

/// Represents errors when utilizing the clipboard abstraction.
#[derive(Debug)]
pub enum ClipboardError {
    /// Failure when initializing the clipboard.
    FailedToInit(String),
    /// Failure to retrieve clipboard content.
    FailedToFetchContent(String),
    /// Failure to set clipboard content.
    FailedToSetContent(String),
}

impl std::error::Error for ClipboardError {}
impl fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClipboardError::FailedToInit(s) => write!(f, "{}", s),
            ClipboardError::FailedToFetchContent(s) => write!(f, "{}", s),
            ClipboardError::FailedToSetContent(s) => write!(f, "{}", s),
        }
    }
}

// Tests
// This doesn't work in CI.
/*#[test]
fn test_clipboard() {
    let mut clipboard = Clipboard::new().unwrap();

    // Preserve user's clipboard contents when testing
    let initial_content = clipboard.get_content().unwrap();

    // Set the content
    let new_content = String::from("Hello, Dioxus!");
    clipboard.set_content(new_content.clone()).unwrap();

    // Get the new content
    let content = clipboard.get_content().unwrap();

    // Return previous content - For some reason this only works if the test panics..?
    clipboard.set_content(initial_content).unwrap();

    // Check if the abstraction worked
    assert_eq!(new_content, content);
}*/
