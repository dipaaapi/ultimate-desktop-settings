//! System layer for Ultimate Windows Desktop Settings.
//!
//! Everything that decides *what* to read or write (registry paths, value
//! formats, catalogs, file layouts) lives in platform-neutral modules and is
//! unit-tested on any OS against [`registry::MemoryRegistry`] and temp dirs.
//! The thin Windows-only layer that actually talks to the OS lives in [`win`]
//! and is exposed through [`system`], which returns
//! [`Error::Unsupported`] on other platforms.

pub mod alarms;
pub mod apps;
pub mod bluetooth;
pub mod browsers;
pub mod config;
pub mod error;
pub mod nightlight;
pub mod notes;
pub mod registry;
pub mod shortcuts;
pub mod startup;
pub mod system;
pub mod temp;
pub mod tweaks;

#[cfg(windows)]
pub mod win;

pub use error::{Error, Result};

/// Folder name used under `%APPDATA%` for notes and settings.
pub const APP_DIR_NAME: &str = "UltimateWindowsDesktopSettings";
