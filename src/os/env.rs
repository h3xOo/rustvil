//! Cross-platform environment variable handling.
//!
//! Provides [`Env`] for safe access to environment variables with proper handling
//! of Windows case-insensitive variables.

use std::collections::HashMap;
use std::ffi::{OsStr, OsString};

use thiserror::Error;

/// Safe wrapper around [`std::env::vars_os`], which is safe to access on Windows: some of its
/// environmental variables are case-insensitive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Env {
    env: HashMap<OsString, OsString>,

    // Map from normalised keys (uppercase) to original.
    normalised_keys: HashMap<String, String>,
}

/// Errors encountered when getting environmental variable.
#[derive(Debug, Clone, Error, PartialEq, Eq, Hash)]
pub enum EnvStrError {
    /// This variant indicates, that variable `Empty.0` is missing.
    #[error("there is no environmental variable `${0:?}`")]
    Missing(OsString),

    /// This variant indicates, that variable `$NonUTF8.0` is not an UTF-8 string.
    #[error("environmental variable `${0:?}` is not an UTF-8 string")]
    NonUTF8(OsString),
}

impl Env {
    /// Create new default [`Env`].
    pub fn new() -> Self {
        Self::new_from(std::env::vars_os().collect())
    }

    /// Create new [`Env`] using `env` as existing environmental variables.
    pub fn new_from(env: HashMap<OsString, OsString>) -> Self {
        let normalised_keys = Env::normalize_keys(&env);
        Self {
            env,
            normalised_keys,
        }
    }

    fn normalize_keys(keys: &HashMap<OsString, OsString>) -> HashMap<String, String> {
        keys.keys()
            .filter_map(|k| k.to_str())
            .map(|k| (k.to_uppercase(), k.to_owned()))
            .collect()
    }

    /// Reload environmental variables from `env`.
    pub fn reload_from(&mut self, env: HashMap<OsString, OsString>) {
        let normalised = Env::normalize_keys(&env);
        self.env = env;
        self.normalised_keys = normalised;
    }

    /// Reload environmental variables from [`std::env::vars_os`].
    pub fn reload(&mut self) {
        self.reload_from(std::env::vars_os().collect())
    }

    /// Get environmental variable pointed by `key`.
    ///
    /// # Arguments
    ///
    /// * `key` - key for environmental variable. Must implement [`AsRef<OsStr>`].
    ///
    /// # Returns
    /// [`Option<&OsStr>`]. [`None`] variant indicates missing key, [`Some`]: existing key.
    ///
    /// # Examples
    /// ```rust
    /// use rustvil::os::env::Env;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let env = Env::new();
    /// println!("$FOO = {:?}", env.get_os("FOO"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_os(&self, key: impl AsRef<OsStr>) -> Option<&OsStr> {
        let key = key.as_ref();
        match self.env.get(key) {
            Some(x) => Some(x),
            None => {
                if cfg!(windows) {
                    self.get_normalised(key)
                } else {
                    None
                }
            }
        }
    }

    fn get_normalised(&self, key: &OsStr) -> Option<&OsStr> {
        let k = key.to_str()?.to_uppercase();
        let env_key: &OsStr = self.normalised_keys.get(&k)?.as_ref();
        self.env.get(env_key).map(OsString::as_ref)
    }

    /// Check, whether this [`Env`] has key `key`.
    pub fn has(&self, key: impl AsRef<OsStr>) -> bool {
        self.get_os(key).is_some()
    }

    /// Get environmental variable pointed by `key` and convert it to UTF-8.
    ///
    /// # Arguments
    ///
    /// * `key` - key for environmental variable. Must implement [`AsRef<Str>`].
    ///
    /// # Returns
    /// [`Result<&str, EnvStrError>`]. [`Ok`] variant indicates existing UTF-8 variable, [`Err`]
    /// indicates some kind of error. See [`EnvStrError`] for
    /// details.
    ///
    /// # Examples
    /// ```rust
    /// use rustvil::os::env::Env;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let env = Env::new();
    /// let _path = env.get("PATH")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn get(&self, key: impl AsRef<OsStr>) -> Result<&str, EnvStrError> {
        let key = key.as_ref();
        self.get_os(key)
            .ok_or_else(|| EnvStrError::Missing(key.to_os_string()))?
            .to_str()
            .ok_or_else(|| EnvStrError::NonUTF8(key.to_os_string()))
    }

    fn from_iter<I: Iterator<Item = (OsString, OsString)>>(t: I) -> Self {
        let mut env = HashMap::new();
        let mut normalised_keys = HashMap::new();
        for (key, value) in t {
            if let Some(key) = key.to_str() {
                normalised_keys.insert(key.to_uppercase(), key.to_owned());
            }
            env.insert(key, value);
        }
        Self {
            env,
            normalised_keys,
        }
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

impl FromIterator<(OsString, OsString)> for Env {
    fn from_iter<T: IntoIterator<Item = (OsString, OsString)>>(iter: T) -> Self {
        Self::from_iter(iter.into_iter())
    }
}

impl<const N: usize> From<[(OsString, OsString); N]> for Env {
    fn from(value: [(OsString, OsString); N]) -> Self {
        <Self as FromIterator<(OsString, OsString)>>::from_iter(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn make_dummy_env() -> Env {
        Env::from([(OsString::from("ala"), OsString::from("bar"))])
    }

    #[test]
    fn basic_test() {
        let env = make_dummy_env();
        assert!(env.has("ala"));
        assert_eq!(env.get_os("ala"), Some(OsStr::new("bar")));
        assert_eq!(env.get("ala"), Ok("bar"));
        if cfg!(windows) {
            assert!(env.has("aLA"));
            assert_eq!(env.get_os("aLA"), Some(OsStr::new("bar")));
            assert_eq!(env.get("aLA"), Ok("bar"));
        } else {
            assert!(!env.has("aLA"));
            assert_eq!(env.get_os("aLA"), None);
            assert_eq!(
                env.get("aLA"),
                Err(EnvStrError::Missing(OsString::from("aLA")))
            );
        }
    }
}
