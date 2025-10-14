//! Configuration file utilities and standard directory paths.
//!
//! Provides helpers for locating configuration files, including XDG Base Directory support.

use std::{env::home_dir, path::PathBuf};

pub mod xdg;

/// Wrapper around [`std::env::home_dir`].
pub fn home() -> Option<PathBuf> {
    home_dir()
}
