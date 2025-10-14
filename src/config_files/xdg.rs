//! XDG Base Directory Specification support.
//!
//! Implements cross-platform path resolution following the XDG Base Directory spec,
//! with platform-specific fallbacks for Windows and macOS.

use crate::os::env::Env;
use std::path::PathBuf;

use crate::config_files::home;

/// How macOS XDG should be treated.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum MacOSBehaviour {
    /// Use fallbacks in `~/Library/...`.
    UseLibrary,
    /// Use fallbacks as Linux ones, like `~/.config/...`.
    LinuxFallback,
}

fn config_fallback(env: &Env, behaviour: MacOSBehaviour) -> Option<PathBuf> {
    if cfg!(windows) {
        env.get("LOCALAPPDATA").ok().map(PathBuf::from)
    } else if cfg!(target_os = "macos") && matches!(behaviour, MacOSBehaviour::UseLibrary) {
        home().map(|mut home| {
            home.push("Library");
            home.push("Application Support");
            home
        })
    } else {
        home().map(|mut home| {
            home.push(".config");
            home
        })
    }
}

fn cache_fallback(env: &Env, behaviour: MacOSBehaviour) -> Option<PathBuf> {
    if cfg!(windows) {
        env.get("LOCALAPPDATA").ok().map(|localappdata| {
            let mut buf = PathBuf::from(localappdata);
            buf.push("caches");
            buf
        })
    } else if cfg!(target_os = "macos") && matches!(behaviour, MacOSBehaviour::UseLibrary) {
        home().map(|mut home| {
            home.push("Library");
            home.push("Caches");
            home
        })
    } else {
        home().map(|mut home| {
            home.push(".cache");
            home
        })
    }
}

fn data_fallback(env: &Env, behaviour: MacOSBehaviour) -> Option<PathBuf> {
    if cfg!(windows) {
        env.get("LOCALAPPDATA").ok().map(PathBuf::from)
    } else if cfg!(target_os = "macos") && matches!(behaviour, MacOSBehaviour::UseLibrary) {
        home().map(|mut home| {
            home.push("Library");
            home.push("Application Support");
            home
        })
    } else {
        home().map(|mut home| {
            home.push(".local");
            home.push(".share");
            home
        })
    }
}

fn state_fallback(env: &Env, behaviour: MacOSBehaviour) -> Option<PathBuf> {
    if cfg!(windows) {
        env.get("LOCALAPPDATA").ok().map(PathBuf::from)
    } else if cfg!(target_os = "macos") && matches!(behaviour, MacOSBehaviour::UseLibrary) {
        home().map(|mut home| {
            home.push("Library");
            home.push("Application Support");
            home
        })
    } else {
        home().map(|mut home| {
            home.push(".local");
            home.push(".state");
            home
        })
    }
}

/// Get proper path for `$XDG_CONFIG_HOME`.
///
/// # Returns
///
/// Most of time it should be [`Some`] variant.
/// [`None`] is returned if and only if:
///     1. [`home`] returns `None`.
///     2. `env` has no key `XDG_CONFIG_HOME`.
pub fn config(env: &Env, behaviour: MacOSBehaviour) -> Option<PathBuf> {
    env.get("XDG_CONFIG_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| config_fallback(env, behaviour))
}

/// Get proper path for `$XDG_DATA_HOME`.
///
/// # Returns
///
/// Most of time it should be [`Some`] variant.
/// [`None`] is returned if and only if:
///     1. [`home`] returns `None`.
///     2. `env` has no key `XDG_DATA_HOME`.
pub fn data(env: &Env, behaviour: MacOSBehaviour) -> Option<PathBuf> {
    env.get("XDG_DATA_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| data_fallback(env, behaviour))
}

/// Get proper path for `$XDG_CACHE_HOME`.
///
/// # Returns
///
/// Most of time it should be [`Some`] variant.
/// [`None`] is returned if and only if:
///     1. [`home`] returns `None`.
///     2. `env` has no key `XDG_CACHE_HOME`.
pub fn cache(env: &Env, behaviour: MacOSBehaviour) -> Option<PathBuf> {
    env.get("XDG_CACHE_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| cache_fallback(env, behaviour))
}
/// Get proper path for `$XDG_STATE_HOME`.
///
/// # Returns
///
/// Most of time it should be [`Some`] variant.
/// [`None`] is returned if and only if:
///     1. [`home`] returns `None`.
///     2. `env` has no key `XDG_STATE_HOME`.
pub fn state(env: &Env, behaviour: MacOSBehaviour) -> Option<PathBuf> {
    env.get("XDG_STATE_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| state_fallback(env, behaviour))
}
