use std::collections::HashMap;
use std::ffi::{OsStr, OsString};

use thiserror::Error;

/// Safe wrapper around [`std::env::vars_os`], which is safe to access on Windows: its
/// environmental variables are case-insensitive.
#[derive(Debug, Clone)]
pub struct Env {
    keys: HashMap<OsString, OsString>,

    normalised_keys: HashMap<OsString, OsString>,
}

/// Errors encountered when getting environmental variable.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
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

    /// Create new [`Env`] using `keys` as existing environmental variables.
    pub fn new_from(env: HashMap<OsString, OsString>) -> Self {
        Self {
            keys: env.clone(),
            normalised_keys: Env::normalize_map(env),
        }
    }

    fn normalize_key(key: impl AsRef<OsStr>) -> OsString {
        key.as_ref().to_ascii_uppercase()
    }
    fn normalize_map(keys: HashMap<OsString, OsString>) -> HashMap<OsString, OsString> {
        keys.into_iter()
            .map(|(key, value)| (Env::normalize_key(key), value))
            .collect()
    }

    /// Reload environmental variables from `env`.
    pub fn reload_from(&mut self, env: HashMap<OsString, OsString>) {
        let normalised = Env::normalize_map(env.clone());
        self.keys = env;
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
    /// * `key` - key for environmental variable. Must implement `AsRef<OsStr>`.
    ///
    /// # Returns
    /// `Option<&OsStr>`. `None` variant indicates missing key, `Some`: existing key.
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
        match self.keys.get(key) {
            Some(x) => Some(x),
            None => {
                if cfg!(target_os = "windows") {
                    self.normalised_keys
                        .get(&Env::normalize_key(key))
                        .map(|x| x.as_ref())
                } else {
                    None
                }
            }
        }
    }

    /// Check, whether this `Env` has key `key`.
    pub fn has(&self, key: impl AsRef<OsStr>) -> bool {
        self.get_os(key).is_some()
    }

    /// Get environmental variable pointed by `key` and convert it to UTF-8.
    ///
    /// # Arguments
    ///
    /// * `key` - key for environmental variable. Must implement `AsRef<Str>`.
    ///
    /// # Returns
    /// `Result<&str, EnvStrError>`. `Ok` variant indicates existing UTF-8 variable, `Err`
    /// indicates some kind of error. See [`EnvStrError`](rustvil::os::env::EnvStrError) for
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
        let mut keys = HashMap::new();
        let mut normalised_keys = HashMap::new();
        for (key, value) in t {
            normalised_keys.insert(Self::normalize_key(&key), value.clone());
            keys.insert(key, value);
        }
        Self {
            keys,
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
        // FIXME: This could be smarter, as we could build both HashMaps at the same time.
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
