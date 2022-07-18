//! URL parsing utils that gracefully handle invalid input.
mod asciiset;
pub use asciiset::AsciiSet;

mod encode;
pub use encode::encode;
