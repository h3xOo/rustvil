use std::{env::home_dir, path::PathBuf};

pub mod xdg;

/// Wrapper around [`std::env::home_dir`].
pub fn home() -> Option<PathBuf> {
    home_dir()
}
