//! Configuration file utilities and standard directory paths.
//!
//! Provides helpers for locating configuration files, including XDG Base Directory support.
//! 
//! ```rust,no_run
//! # use rustvil::config_files::xdg::{self, MacOSBehaviour};
//! # use rustvil::os::env::Env;
//! # fn foo() -> Option<()> {
//! let env = Env::new();
//! 
//! let config_path = xdg::config(&env, MacOSBehaviour::LinuxFallback)?;
//! # None
//! # }
//! ```

use std::{env::home_dir, path::PathBuf};

pub mod xdg;

/// Wrapper around [`std::env::home_dir`].
pub fn home() -> Option<PathBuf> {
    home_dir()
}
