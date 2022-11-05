//! Provides a clipboard abstraction to access the target system's clipboard.

use clipboard::{ClipboardContext, ClipboardProvider};

use crate::DioxusStdError;

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
    pub fn new() -> Result<Self, DioxusStdError> {
        let ctx: ClipboardContext = match ClipboardProvider::new() {
            Ok(ctx) => ctx,
            Err(e) => return Err(DioxusStdError::Clipboard(e.to_string())),
        };

        Ok(Self { ctx })
    }

    /// Provides a [`String`] of the target system's current clipboard content.
    pub fn get_contents(&mut self) -> Result<String, DioxusStdError> {
        match self.ctx.get_contents() {
            Ok(content) => Ok(content),
            Err(e) => return Err(DioxusStdError::Clipboard(e.to_string())),
        }
    }

    /// Set the clipboard's content to the provided [`String`]
    pub fn set_content(&mut self, value: String) -> Result<(), DioxusStdError> {
        match self.ctx.set_contents(value) {
            Ok(()) => Ok(()),
            Err(e) => Err(DioxusStdError::Clipboard(e.to_string())),
        }
    }
}


#[test]
fn test_clipboard() {
    let mut clipboard = Clipboard::new().unwrap();

    // Preserve user's clipboard contents when testing
    let initial_content = clipboard.get_contents().unwrap();
    
    // Set the content
    let new_content = String::from("Hello, Dioxus!");
    clipboard.set_content(new_content.clone()).unwrap();

    // Get the new content
    let content = clipboard.get_contents().unwrap();
    
    // Return previous content - For some reason this only works if the test panics..?
    clipboard.set_content(initial_content).unwrap(); 

    // Check if the abstraction worked
    assert_eq!(new_content, content);
}