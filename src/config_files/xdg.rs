use crate::os::env::Env;
use std::path::PathBuf;

/// How macOS XDG should be treated.
#[derive(Debug, Clone, Copy, Hash)]
pub enum MacOSBehaviour {
    /// Use fallbacks in `~/Library/...`.
    UseLibrary,
    /// Use fallbacks as Linux does, like `~/.config/...`.
    LinuxFallback,
}

fn config_fallback(env: &Env, behaviour: MacOSBehaviour) -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        env.get("LOCALAPPDATA").ok().map(PathBuf::from)
    } else if cfg!(target_os = "macos") && matches!(behaviour, MacOSBehaviour::UseLibrary) {
        None
    } else {
        None
    }
}
