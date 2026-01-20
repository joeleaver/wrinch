//! Cross-platform clipboard support for text and images.
//!
//! This module provides clipboard operations using the `arboard` crate.
//!
//! # Example
//!
//! ```ignore
//! use rinch::clipboard::{copy_text, paste_text, has_text};
//!
//! // Copy text to clipboard
//! copy_text("Hello, clipboard!").unwrap();
//!
//! // Check if clipboard has text
//! if has_text() {
//!     // Paste text from clipboard
//!     if let Ok(text) = paste_text() {
//!         println!("Clipboard: {}", text);
//!     }
//! }
//! ```

use arboard::Clipboard;
use std::sync::Mutex;

/// Clipboard error type.
#[derive(Debug)]
pub enum ClipboardError {
    /// Failed to access the clipboard.
    AccessFailed(String),
    /// The clipboard doesn't contain the expected content type.
    ContentTypeMismatch,
}

impl std::fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipboardError::AccessFailed(msg) => write!(f, "clipboard access failed: {}", msg),
            ClipboardError::ContentTypeMismatch => write!(f, "clipboard content type mismatch"),
        }
    }
}

impl std::error::Error for ClipboardError {}

impl From<arboard::Error> for ClipboardError {
    fn from(err: arboard::Error) -> Self {
        ClipboardError::AccessFailed(err.to_string())
    }
}

/// Result type for clipboard operations.
pub type ClipboardResult<T> = Result<T, ClipboardError>;

// Global clipboard instance (arboard recommends reusing the Clipboard instance)
static CLIPBOARD: Mutex<Option<Clipboard>> = Mutex::new(None);

fn with_clipboard<T, F: FnOnce(&mut Clipboard) -> ClipboardResult<T>>(f: F) -> ClipboardResult<T> {
    let mut guard = CLIPBOARD.lock().unwrap();
    if guard.is_none() {
        *guard = Some(Clipboard::new()?);
    }
    f(guard.as_mut().unwrap())
}

/// Copy text to the clipboard.
///
/// # Example
///
/// ```ignore
/// use rinch::clipboard::copy_text;
///
/// copy_text("Hello, world!").unwrap();
/// ```
pub fn copy_text(text: impl AsRef<str>) -> ClipboardResult<()> {
    with_clipboard(|clipboard| {
        clipboard.set_text(text.as_ref())?;
        Ok(())
    })
}

/// Paste text from the clipboard.
///
/// Returns `Err` if the clipboard doesn't contain text.
///
/// # Example
///
/// ```ignore
/// use rinch::clipboard::paste_text;
///
/// match paste_text() {
///     Ok(text) => println!("Pasted: {}", text),
///     Err(e) => println!("Failed to paste: {}", e),
/// }
/// ```
pub fn paste_text() -> ClipboardResult<String> {
    with_clipboard(|clipboard| {
        let text = clipboard.get_text()?;
        Ok(text)
    })
}

/// Check if the clipboard contains text.
///
/// # Example
///
/// ```ignore
/// use rinch::clipboard::has_text;
///
/// if has_text() {
///     println!("Clipboard has text");
/// }
/// ```
pub fn has_text() -> bool {
    paste_text().is_ok()
}

/// Clear the clipboard contents.
///
/// # Example
///
/// ```ignore
/// use rinch::clipboard::clear;
///
/// clear().unwrap();
/// ```
pub fn clear() -> ClipboardResult<()> {
    with_clipboard(|clipboard| {
        clipboard.clear()?;
        Ok(())
    })
}

/// Copy an image to the clipboard.
///
/// The image data should be in RGBA format.
///
/// # Example
///
/// ```ignore
/// use rinch::clipboard::{copy_image, ImageData};
///
/// let image = ImageData {
///     width: 100,
///     height: 100,
///     bytes: vec![255; 100 * 100 * 4].into(), // White image
/// };
/// copy_image(image).unwrap();
/// ```
pub fn copy_image(image: ImageData) -> ClipboardResult<()> {
    with_clipboard(|clipboard| {
        let arboard_image = arboard::ImageData {
            width: image.width,
            height: image.height,
            bytes: image.bytes,
        };
        clipboard.set_image(arboard_image)?;
        Ok(())
    })
}

/// Paste an image from the clipboard.
///
/// Returns the image data in RGBA format.
///
/// # Example
///
/// ```ignore
/// use rinch::clipboard::paste_image;
///
/// if let Ok(image) = paste_image() {
///     println!("Image: {}x{}", image.width, image.height);
/// }
/// ```
pub fn paste_image() -> ClipboardResult<ImageData<'static>> {
    with_clipboard(|clipboard| {
        let image = clipboard.get_image()?;
        Ok(ImageData {
            width: image.width,
            height: image.height,
            bytes: image.bytes,
        })
    })
}

/// Check if the clipboard contains an image.
///
/// # Example
///
/// ```ignore
/// use rinch::clipboard::has_image;
///
/// if has_image() {
///     println!("Clipboard has an image");
/// }
/// ```
pub fn has_image() -> bool {
    paste_image().is_ok()
}

/// Image data for clipboard operations.
///
/// The bytes are in RGBA format (4 bytes per pixel).
#[derive(Debug, Clone)]
pub struct ImageData<'a> {
    /// Width in pixels.
    pub width: usize,
    /// Height in pixels.
    pub height: usize,
    /// RGBA pixel data.
    pub bytes: std::borrow::Cow<'a, [u8]>,
}

impl<'a> ImageData<'a> {
    /// Create new image data.
    pub fn new(width: usize, height: usize, bytes: impl Into<std::borrow::Cow<'a, [u8]>>) -> Self {
        Self {
            width,
            height,
            bytes: bytes.into(),
        }
    }

    /// Convert to owned data.
    pub fn into_owned(self) -> ImageData<'static> {
        ImageData {
            width: self.width,
            height: self.height,
            bytes: std::borrow::Cow::Owned(self.bytes.into_owned()),
        }
    }
}
