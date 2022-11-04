

use clipboard::{ClipboardContext, ClipboardProvider};

use crate::DioxusStdError;

/// Provides a clipboard abstraction to access the target system's clipboard.
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
