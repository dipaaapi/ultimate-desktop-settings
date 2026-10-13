//! The Windows-only layer. Each function here does one OS call (or a small
//! group of them); the decisions about what to call live in the parent crate.

pub mod audio;
pub mod display;
pub mod process;
pub mod registry;
pub mod service;
pub mod shell;

/// Converts a Rust string to a null-terminated UTF-16 buffer.
pub(crate) fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
