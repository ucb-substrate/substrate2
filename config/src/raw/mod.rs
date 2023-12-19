//! Raw string configuration utilities.
//!
//! The `RawConfig` object contains general information about the environment,
//! and provides access to Substrate's configuration files.
//!
//! ## Config value API
//!
//! The primary API for fetching user-defined config values is the
//! `RawConfig::get` method. It uses `serde` to translate config values to a
//! target type.
//!
//! There are a variety of helper types for deserializing some common formats:
//!
//! - `value::Value`: This type provides access to the location where the
//!   config value was defined.
//! - `ConfigRelativePath`: For a path that is relative to where it is
//!   defined.
//! - `PathAndArgs`: Similar to `ConfigRelativePath`, but also supports a list
//!   of arguments, useful for programs to execute.
//! - `StringList`: Get a value that is either a list or a whitespace split
//!   string.
//!
//! ## Map key recommendations
//!
//! Handling tables that have arbitrary keys can be tricky, particularly if it
//! should support environment variables. In general, if possible, the caller
//! should pass the full key path into the `get()` method so that the config
//! deserializer can properly handle environment variables (which need to be
//! uppercased, and dashes converted to underscores).
//!
//! Try to avoid keys that are a prefix of another with a dash/underscore. For
//! example `build.target` and `build.target-dir`. This is OK if these are not
//! structs/maps, but if it is a struct or map, then it will not be able to
//! read the environment variable due to ambiguity. (See `ConfigMapAccess` for
//! more details.)
//!
//! ## Internal API
//!
//! Internally config values are stored with the `ConfigValue` type after they
//! have been loaded from disk. This is similar to the `toml::Value` type, but
//! includes the definition location. The `get()` method uses serde to
//! translate from `ConfigValue` and environment variables to the caller's
//! desired type.
//!
//
// ## LICENSING
//
// Based on Cargo's [`config` module](https://github.com/rust-lang/cargo/tree/master/src/cargo/util/config)
// with substantial modifications.

use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, HashSet};
use std::env;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs;
use std::mem;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::Result;

use self::ConfigValue as CV;
use crate::paths;
use anyhow::{anyhow, bail, Context as _};
use lazycell::LazyCell;
use serde::Deserialize;

use de::Deserializer;
pub(crate) use environment::Env;
pub(crate) use key::ConfigKey;
pub(crate) use path::ConfigRelativePath;
pub(crate) use value::{Definition, OptValue, Value};

mod de;
mod environment;
mod key;
mod path;
mod value;

// Helper macro for creating typed access methods.
macro_rules! get_value_typed {
    ($name:ident, $ty:ty, $variant:ident, $expected:expr) => {
        /// Low-level private method for getting a config value as an OptValue.
        fn $name(&self, key: &ConfigKey) -> Result<OptValue<$ty>, ConfigError> {
            let cv = self.get_cv(key)?;
            let env = self.get_config_env::<$ty>(key)?;
            match (cv, env) {
                (Some(CV::$variant(val, definition)), Some(env)) => {
                    if definition.is_higher_priority(&env.definition) {
                        Ok(Some(Value { val, definition }))
                    } else {
                        Ok(Some(env))
                    }
                }
                (Some(CV::$variant(val, definition)), None) => Ok(Some(Value { val, definition })),
                (Some(cv), _) => Err(ConfigError::expected(key, $expected, &cv)),
                (None, Some(env)) => Ok(Some(env)),
                (None, None) => Ok(None),
            }
        }
    };
}

/// Configuration information for Substrate.
#[derive(Debug)]
pub(crate) struct RawConfig {
    /// The location of the user's Substrate home directory. OS-dependent.
    home_path: PathBuf,
    /// A collection of configuration options
    values: LazyCell<HashMap<String, ConfigValue>>,
    /// The current working directory of Substrate
    cwd: PathBuf,
    /// Directory where config file searching should stop (inclusive).
    search_stop_path: Option<PathBuf>,
    /// Environment variable snapshot.
    env: Env,
}

impl RawConfig {
    /// Creates a new [`RawConfig`] instance.
    ///
    /// This is typically used for tests or other special cases. `default` is
    /// preferred otherwise.
    ///
    /// This does only minimal initialization. In particular, it does not load
    /// any config files from disk. Those will be loaded lazily as-needed.
    pub(crate) fn new(cwd: PathBuf, homedir: PathBuf) -> RawConfig {
        RawConfig {
            home_path: homedir,
            cwd,
            search_stop_path: None,
            values: LazyCell::new(),
            env: Env::new(),
        }
    }

    /// Creates a new [`RawConfig`] instance, with all default settings.
    ///
    /// This does only minimal initialization. In particular, it does not load
    /// any config files from disk. Those will be loaded lazily as-needed.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn default() -> Result<RawConfig> {
        let cwd = env::current_dir()
            .with_context(|| "couldn't get the current directory of the process")?;
        let homedir = homedir(&cwd).ok_or_else(|| {
            anyhow!(
                "Substrate couldn't find your home directory. \
                 This probably means that $HOME was not set."
            )
        })?;
        Ok(RawConfig::new(cwd, homedir))
    }

    /// Gets the user's Substrate home directory (OS-dependent).
    #[allow(dead_code)]
    pub(crate) fn home(&self) -> &PathBuf {
        &self.home_path
    }

    /// Returns a path to display to the user with the location of their home
    /// config file (to only be used for displaying a diagnostics suggestion,
    /// such as recommending where to add a config value).
    #[allow(dead_code)]
    pub(crate) fn diagnostic_home_config(&self) -> String {
        let home = self.home_path.clone();
        let path = home.join("config.toml");
        path.to_string_lossy().to_string()
    }

    /// Gets all config values from disk.
    ///
    /// This will lazy-load the values as necessary. Callers are responsible
    /// for checking environment variables. Callers outside of the `config`
    /// module should avoid using this.
    fn values(&self) -> Result<&HashMap<String, ConfigValue>> {
        self.values.try_borrow_with(|| self.load_values())
    }

    /// Sets the path where ancestor config file searching will stop. The
    /// given path is included, but its ancestors are not.
    #[allow(dead_code)]
    pub(crate) fn set_search_stop_path<P: Into<PathBuf>>(&mut self, path: P) {
        let path = path.into();
        debug_assert!(self.cwd.starts_with(&path));
        self.search_stop_path = Some(path);
    }

    /// Reloads on-disk configuration values, starting at the given path and
    /// walking up its ancestors.
    #[allow(dead_code)]
    pub(crate) fn reload_rooted_at<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let values = self.load_values_from(path.as_ref())?;
        self.values.replace(values);
        Ok(())
    }

    /// The current working directory.
    pub(crate) fn cwd(&self) -> &Path {
        &self.cwd
    }

    /// Get a configuration value by key.
    ///
    /// This does NOT look at environment variables. See [`RawConfig::get_cv_with_env`] for
    /// a variant that supports environment variables.
    fn get_cv(&self, key: &ConfigKey) -> Result<Option<ConfigValue>> {
        self.get_cv_helper(key, self.values()?)
    }

    fn get_cv_helper(
        &self,
        key: &ConfigKey,
        vals: &HashMap<String, ConfigValue>,
    ) -> Result<Option<ConfigValue>> {
        log::trace!("get cv {:?}", key);
        if key.is_root() {
            // Returning the entire root table.
            return Ok(Some(CV::Table(
                vals.clone(),
                Definition::Path(PathBuf::new()),
            )));
        }
        let mut parts = key.parts().enumerate();
        let mut val = match vals.get(parts.next().unwrap().1) {
            Some(val) => val,
            None => return Ok(None),
        };
        for (i, part) in parts {
            match val {
                CV::Table(map, _) => {
                    val = match map.get(part) {
                        Some(val) => val,
                        None => return Ok(None),
                    }
                }
                CV::Integer(_, def)
                | CV::String(_, def)
                | CV::List(_, def)
                | CV::Boolean(_, def) => {
                    let mut key_so_far = ConfigKey::new();
                    for part in key.parts().take(i) {
                        key_so_far.push(part);
                    }
                    bail!(
                        "expected table for configuration key `{}`, \
                         but found {} in {}",
                        key_so_far,
                        val.desc(),
                        def
                    )
                }
            }
        }
        Ok(Some(val.clone()))
    }

    /// This is a helper for getting a CV from a file or env var.
    pub(crate) fn get_cv_with_env(&self, key: &ConfigKey) -> Result<Option<CV>> {
        // Determine if value comes from env, cli, or file, and merge env if
        // possible.
        let cv = self.get_cv(key)?;
        if key.is_root() {
            // Root table can't have env value.
            return Ok(cv);
        }
        let env = self.env.get_str(key.as_env_key());
        let env_def = Definition::Environment(key.as_env_key().to_string());
        let use_env = match (&cv, env) {
            // Lists are always merged.
            (Some(CV::List(..)), Some(_)) => true,
            (Some(cv), Some(_)) => env_def.is_higher_priority(cv.definition()),
            (None, Some(_)) => true,
            _ => false,
        };

        if !use_env {
            return Ok(cv);
        }

        // Future note: If you ever need to deserialize a non-self describing
        // map type, this should implement a starts_with check (similar to how
        // ConfigMapAccess does).
        let env = env.unwrap();
        if env == "true" {
            Ok(Some(CV::Boolean(true, env_def)))
        } else if env == "false" {
            Ok(Some(CV::Boolean(false, env_def)))
        } else if let Ok(i) = env.parse::<i64>() {
            Ok(Some(CV::Integer(i, env_def)))
        } else {
            // Try to merge if possible.
            match cv {
                Some(CV::List(mut cv_list, cv_def)) => {
                    // Merge with config file.
                    self.get_env_list(key, &mut cv_list)?;
                    Ok(Some(CV::List(cv_list, cv_def)))
                }
                _ => {
                    // Note: CV::Table merging is not implemented, as env
                    // vars do not support table values. In the future, we
                    // could check for `{}`, and interpret it as TOML if
                    // that seems useful.
                    Ok(Some(CV::String(env.to_string(), env_def)))
                }
            }
        }
    }

    /// Helper for testing.
    #[cfg(test)]
    pub(crate) fn set_env(&mut self, env: HashMap<String, String>) {
        self.env = Env::from_map(env);
    }

    /// Returns all environment variable keys, filtering out keys that are not valid UTF-8.
    fn env_keys(&self) -> impl Iterator<Item = &str> {
        self.env.keys_str()
    }

    fn get_config_env<T>(&self, key: &ConfigKey) -> Result<OptValue<T>, ConfigError>
    where
        T: FromStr,
        <T as FromStr>::Err: fmt::Display,
    {
        match self.env.get_str(key.as_env_key()) {
            Some(value) => {
                let definition = Definition::Environment(key.as_env_key().to_string());
                Ok(Some(Value {
                    val: value
                        .parse()
                        .map_err(|e| ConfigError::new(format!("{}", e), definition.clone()))?,
                    definition,
                }))
            }
            None => {
                // self.check_environment_key_case_mismatch(key);
                Ok(None)
            }
        }
    }

    /// Get the value of environment variable `key` through the [`RawConfig`] snapshot.
    ///
    /// This can be used similarly to `std::env::var`.
    #[allow(dead_code)]
    pub(crate) fn get_env(&self, key: impl AsRef<OsStr>) -> Result<String> {
        self.env.get_env(key)
    }

    /// Get the value of environment variable `key` through the [`RawConfig`] snapshot.
    ///
    /// This can be used similarly to `std::env::var_os`.
    #[allow(dead_code)]
    pub(crate) fn get_env_os(&self, key: impl AsRef<OsStr>) -> Option<OsString> {
        self.env.get_env_os(key)
    }

    /// Check if the [`RawConfig`] contains a given [`ConfigKey`].
    ///
    /// See `ConfigMapAccess` for a description of `env_prefix_ok`.
    fn has_key(&self, key: &ConfigKey, env_prefix_ok: bool) -> Result<bool> {
        if self.env.contains_key(key.as_env_key()) {
            return Ok(true);
        }
        if env_prefix_ok {
            let env_prefix = format!("{}_", key.as_env_key());
            if self.env_keys().any(|k| k.starts_with(&env_prefix)) {
                return Ok(true);
            }
        }
        if self.get_cv(key)?.is_some() {
            return Ok(true);
        }
        self.check_environment_key_case_mismatch(key);

        Ok(false)
    }

    fn check_environment_key_case_mismatch(&self, key: &ConfigKey) {
        if let Some(_env_key) = self.env.get_normalized(key.as_env_key()) {
            // TODO: Implement warnings.
            // let _ = self.shell().warn(format!(
            //     "Environment variables are expected to use uppercase letters and underscores, \
            //     the variable `{}` will be ignored and have no effect",
            //     env_key
            // ));
        }
    }

    /// Get a string config value.
    ///
    /// See [`RawConfig::get`] for more details.
    #[allow(dead_code)]
    pub(crate) fn get_string(&self, key: &str) -> Result<OptValue<String>> {
        self.get::<Option<Value<String>>>(key)
    }

    /// Get a config value that is expected to be a path.
    ///
    /// This returns a relative path if the value does not contain any
    /// directory separators. See [`ConfigRelativePath::resolve_program`] for
    /// more details.
    #[allow(dead_code)]
    pub(crate) fn get_path(&self, key: &str) -> Result<OptValue<PathBuf>> {
        self.get::<Option<Value<ConfigRelativePath>>>(key).map(|v| {
            v.map(|v| Value {
                val: v.val.resolve_program(self),
                definition: v.definition,
            })
        })
    }

    #[allow(dead_code)]
    fn string_to_path(&self, value: &str, definition: &Definition) -> PathBuf {
        let is_path = value.contains('/') || (cfg!(windows) && value.contains('\\'));
        if is_path {
            definition.root(self).join(value)
        } else {
            // A pathless name.
            PathBuf::from(value)
        }
    }

    /// Get a list of strings.
    ///
    /// NOTE: this does **not** support environment variables. Use [`RawConfig::get`] instead
    /// if you want that.
    fn _get_list(&self, key: &ConfigKey) -> Result<OptValue<Vec<(String, Definition)>>> {
        match self.get_cv(key)? {
            Some(CV::List(val, definition)) => Ok(Some(Value { val, definition })),
            Some(val) => self.expected("list", key, &val),
            None => Ok(None),
        }
    }

    /// Helper for StringList type to get something that is a string or list.
    fn get_list_or_string(
        &self,
        key: &ConfigKey,
        merge: bool,
    ) -> Result<Vec<(String, Definition)>> {
        let mut res = Vec::new();

        if !merge {
            self.get_env_list(key, &mut res)?;

            if !res.is_empty() {
                return Ok(res);
            }
        }

        match self.get_cv(key)? {
            Some(CV::List(val, _def)) => res.extend(val),
            Some(CV::String(val, def)) => {
                let split_vs = val.split_whitespace().map(|s| (s.to_string(), def.clone()));
                res.extend(split_vs);
            }
            Some(val) => {
                return self.expected("string or array of strings", key, &val);
            }
            None => {}
        }

        self.get_env_list(key, &mut res)?;

        Ok(res)
    }

    /// Internal method for getting an environment variable as a list.
    fn get_env_list(&self, key: &ConfigKey, output: &mut Vec<(String, Definition)>) -> Result<()> {
        let env_val = match self.env.get_str(key.as_env_key()) {
            Some(v) => v,
            None => {
                // self.check_environment_key_case_mismatch(key);
                return Ok(());
            }
        };

        let def = Definition::Environment(key.as_env_key().to_string());
        output.extend(
            env_val
                .split_whitespace()
                .map(|s| (s.to_string(), def.clone())),
        );
        Ok(())
    }

    /// Low-level method for getting a config value as an `OptValue<HashMap<String, CV>>`.
    ///
    /// NOTE: This does not read from env. The caller is responsible for that.
    fn get_table(&self, key: &ConfigKey) -> Result<OptValue<HashMap<String, CV>>> {
        match self.get_cv(key)? {
            Some(CV::Table(val, definition)) => Ok(Some(Value { val, definition })),
            Some(val) => self.expected("table", key, &val),
            None => Ok(None),
        }
    }

    get_value_typed! {get_integer, i64, Integer, "an integer"}
    get_value_typed! {get_bool, bool, Boolean, "true/false"}
    get_value_typed! {get_string_priv, String, String, "a string"}

    /// Generate an error when the given value is the wrong type.
    fn expected<T>(&self, ty: &str, key: &ConfigKey, val: &CV) -> Result<T> {
        val.expected(ty, &key.to_string())
            .map_err(|e| anyhow!("invalid configuration for key `{}`\n{}", key, e))
    }

    /// Loads configuration from the filesystem.
    pub(crate) fn load_values(&self) -> Result<HashMap<String, ConfigValue>> {
        self.load_values_from(&self.cwd)
    }

    /// Start a config file discovery from a path and merges all config values found.
    fn load_values_from(&self, path: &Path) -> Result<HashMap<String, ConfigValue>> {
        // This definition path is ignored, this is just a temporary container
        // representing the entire file.
        let mut cfg = CV::Table(HashMap::new(), Definition::Path(PathBuf::from(".")));
        let home = self.home_path.clone();

        self.walk_tree(path, &home, |path| {
            let value = self.load_file(path)?;
            cfg.merge(value, false).with_context(|| {
                format!("failed to merge configuration at `{}`", path.display())
            })?;
            Ok(())
        })
        .with_context(|| "could not load Substrate configuration")?;

        match cfg {
            CV::Table(map, _) => Ok(map),
            _ => unreachable!(),
        }
    }

    /// Loads a config value from a path.
    ///
    /// This is used during config file discovery.
    fn load_file(&self, path: &Path) -> Result<ConfigValue> {
        self._load_file(path, &mut HashSet::new(), true)
    }

    /// Loads a config value from a path with options.
    ///
    /// This is actual implementation of loading a config value from a path.
    ///
    /// * `includes` determines whether to load configs from [`config-include`].
    /// * `seen` is used to check for cyclic includes.
    /// * `why_load` tells why a config is being loaded.
    ///
    /// [`config-include`]: https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#config-include
    fn _load_file(
        &self,
        path: &Path,
        seen: &mut HashSet<PathBuf>,
        includes: bool,
    ) -> Result<ConfigValue> {
        if !seen.insert(path.to_path_buf()) {
            bail!(
                "config `include` cycle detected with path `{}`",
                path.display()
            );
        }
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read configuration file `{}`", path.display()))?;
        let toml = contents
            .parse()
            .map_err(|e| anyhow::Error::from(e).context("could not parse input as TOML"))
            .with_context(|| {
                format!("could not parse TOML configuration in `{}`", path.display())
            })?;
        let def = Definition::Path(path.into());
        let value = CV::from_toml(def, toml::Value::Table(toml)).with_context(|| {
            format!(
                "failed to load TOML configuration from `{}`",
                path.display()
            )
        })?;
        if includes {
            self.load_includes(value, seen)
        } else {
            Ok(value)
        }
    }

    /// Load any `include` files listed in the given `value`.
    ///
    /// Returns `value` with the given include files merged into it.
    ///
    /// * `seen` is used to check for cyclic includes.
    /// * `why_load` tells why a config is being loaded.
    fn load_includes(&self, mut value: CV, seen: &mut HashSet<PathBuf>) -> Result<CV> {
        // Get the list of files to load.
        let includes = self.include_paths(&mut value, true)?;
        // Accumulate all values here.
        let mut root = CV::Table(HashMap::new(), value.definition().clone());
        for (path, abs_path, def) in includes {
            self._load_file(&abs_path, seen, true)
                .and_then(|include| root.merge(include, true))
                .with_context(|| {
                    format!("failed to load config include `{}` from `{}`", path, def)
                })?;
        }
        root.merge(value, true)?;
        Ok(root)
    }

    /// Converts the `include` config value to a list of absolute paths.
    fn include_paths(
        &self,
        cv: &mut CV,
        remove: bool,
    ) -> Result<Vec<(String, PathBuf, Definition)>> {
        let abs = |path: &str, def: &Definition| -> (String, PathBuf, Definition) {
            let abs_path = match def {
                Definition::Path(p) => p.parent().unwrap().join(path),
                Definition::Environment(_) => self.cwd().join(path),
            };
            (path.to_string(), abs_path, def.clone())
        };
        let table = match cv {
            CV::Table(table, _def) => table,
            _ => unreachable!(),
        };
        let owned;
        let include = if remove {
            owned = table.remove("include");
            owned.as_ref()
        } else {
            table.get("include")
        };
        let includes = match include {
            Some(CV::String(s, def)) => {
                vec![abs(s, def)]
            }
            Some(CV::List(list, _def)) => list.iter().map(|(s, def)| abs(s, def)).collect(),
            Some(other) => bail!(
                "`include` expected a string or list, but found {} in `{}`",
                other.desc(),
                other.definition()
            ),
            None => {
                return Ok(Vec::new());
            }
        };
        Ok(includes)
    }

    fn walk_tree<F>(&self, pwd: &Path, home: &Path, mut walk: F) -> Result<()>
    where
        F: FnMut(&Path) -> Result<()>,
    {
        let mut stash: HashSet<PathBuf> = HashSet::new();

        for current in paths::ancestors(pwd, self.search_stop_path.as_deref()) {
            let path = current.join(".substrate").join("config.toml");
            if path.exists() {
                walk(&path)?;
                stash.insert(path);
            }
        }

        // Once we're done, also be sure to walk the home directory even if it's not
        // in our history to be sure we pick up that standard location for
        // information.
        let path = home.join("config.toml");
        if path.exists() && !stash.contains(&path) {
            walk(&path)?;
        }

        Ok(())
    }

    /// Retrieves a config variable.
    ///
    /// This supports most serde `Deserialize` types. Examples:
    ///
    /// ```rust,ignore
    /// let v: Option<u32> = config.get("some.nested.key")?;
    /// let v: Option<MyStruct> = config.get("some.key")?;
    /// let v: Option<HashMap<String, MyStruct>> = config.get("foo")?;
    /// ```
    ///
    /// The key may be a dotted key, but this does NOT support TOML key
    /// quoting. Avoid key components that may have dots. For example,
    /// `foo.'a.b'.bar" does not work if you try to fetch `foo.'a.b'". You can
    /// fetch `foo` if it is a map, though.
    pub(crate) fn get<'de, T: serde::de::Deserialize<'de>>(&self, key: &str) -> Result<T> {
        let d = Deserializer {
            config: self,
            key: ConfigKey::from_str(key),
            env_prefix_ok: true,
        };
        T::deserialize(d).map_err(|e| e.into())
    }
}

/// Internal error for serde errors.
#[derive(Debug)]
pub(crate) struct ConfigError {
    error: anyhow::Error,
    definition: Option<Definition>,
}

impl ConfigError {
    fn new(message: String, definition: Definition) -> ConfigError {
        ConfigError {
            error: anyhow::Error::msg(message),
            definition: Some(definition),
        }
    }

    fn expected(key: &ConfigKey, expected: &str, found: &ConfigValue) -> ConfigError {
        ConfigError {
            error: anyhow!(
                "`{}` expected {}, but found a {}",
                key,
                expected,
                found.desc()
            ),
            definition: Some(found.definition().clone()),
        }
    }

    fn missing(key: &ConfigKey) -> ConfigError {
        ConfigError {
            error: anyhow!("missing config key `{}`", key),
            definition: None,
        }
    }

    fn with_key_context(self, key: &ConfigKey, definition: Definition) -> ConfigError {
        ConfigError {
            error: anyhow::Error::from(self)
                .context(format!("could not load config key `{}`", key)),
            definition: Some(definition),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.error.source()
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(definition) = &self.definition {
            write!(f, "error in {}: {}", definition, self.error)
        } else {
            self.error.fmt(f)
        }
    }
}

impl serde::de::Error for ConfigError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        ConfigError {
            error: anyhow::Error::msg(msg.to_string()),
            definition: None,
        }
    }
}

impl From<anyhow::Error> for ConfigError {
    fn from(error: anyhow::Error) -> Self {
        ConfigError {
            error,
            definition: None,
        }
    }
}

#[derive(Eq, PartialEq, Clone)]
/// A configuration value to deserialize.
pub(crate) enum ConfigValue {
    /// An integer configuration value.
    Integer(i64, Definition),
    /// A string configuration value.
    String(String, Definition),
    /// A list of string configuration values.
    List(Vec<(String, Definition)>, Definition),
    /// A table configuration value mapping strings to
    /// additional configuration values.
    Table(HashMap<String, ConfigValue>, Definition),
    /// A boolean configuration value.
    Boolean(bool, Definition),
}

impl fmt::Debug for ConfigValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CV::Integer(i, def) => write!(f, "{} (from {})", i, def),
            CV::Boolean(b, def) => write!(f, "{} (from {})", b, def),
            CV::String(s, def) => write!(f, "{} (from {})", s, def),
            CV::List(list, def) => {
                write!(f, "[")?;
                for (i, (s, def)) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{} (from {})", s, def)?;
                }
                write!(f, "] (from {})", def)
            }
            CV::Table(table, _) => write!(f, "{:?}", table),
        }
    }
}

impl ConfigValue {
    fn from_toml(def: Definition, toml: toml::Value) -> Result<ConfigValue> {
        match toml {
            toml::Value::String(val) => Ok(CV::String(val, def)),
            toml::Value::Boolean(b) => Ok(CV::Boolean(b, def)),
            toml::Value::Integer(i) => Ok(CV::Integer(i, def)),
            toml::Value::Array(val) => Ok(CV::List(
                val.into_iter()
                    .map(|toml| match toml {
                        toml::Value::String(val) => Ok((val, def.clone())),
                        v => bail!("expected string but found {} in list", v.type_str()),
                    })
                    .collect::<Result<_>>()?,
                def,
            )),
            toml::Value::Table(val) => Ok(CV::Table(
                val.into_iter()
                    .map(|(key, value)| {
                        let value = CV::from_toml(def.clone(), value)
                            .with_context(|| format!("failed to parse key `{}`", key))?;
                        Ok((key, value))
                    })
                    .collect::<Result<_>>()?,
                def,
            )),
            v => bail!(
                "found TOML configuration value of unknown type `{}`",
                v.type_str()
            ),
        }
    }

    #[allow(dead_code)]
    fn into_toml(self) -> toml::Value {
        match self {
            CV::Boolean(s, _) => toml::Value::Boolean(s),
            CV::String(s, _) => toml::Value::String(s),
            CV::Integer(i, _) => toml::Value::Integer(i),
            CV::List(l, _) => {
                toml::Value::Array(l.into_iter().map(|(s, _)| toml::Value::String(s)).collect())
            }
            CV::Table(l, _) => {
                toml::Value::Table(l.into_iter().map(|(k, v)| (k, v.into_toml())).collect())
            }
        }
    }

    /// Merge the given value into self.
    ///
    /// If `force` is true, primitive (non-container) types will override existing values.
    /// If false, the original will be kept and the new value ignored.
    ///
    /// Container types (tables and arrays) are merged with existing values.
    ///
    /// Container and non-container types cannot be mixed.
    fn merge(&mut self, from: ConfigValue, force: bool) -> Result<()> {
        match (self, from) {
            (&mut CV::List(ref mut old, _), CV::List(ref mut new, _)) => {
                old.extend(mem::take(new));
            }
            (&mut CV::Table(ref mut old, _), CV::Table(ref mut new, _)) => {
                for (key, value) in mem::take(new) {
                    match old.entry(key.clone()) {
                        Occupied(mut entry) => {
                            let new_def = value.definition().clone();
                            let entry = entry.get_mut();
                            entry.merge(value, force).with_context(|| {
                                format!(
                                    "failed to merge key `{}` between \
                                     {} and {}",
                                    key,
                                    entry.definition(),
                                    new_def,
                                )
                            })?;
                        }
                        Vacant(entry) => {
                            entry.insert(value);
                        }
                    };
                }
            }
            // Allow switching types except for tables or arrays.
            (expected @ &mut CV::List(_, _), found)
            | (expected @ &mut CV::Table(_, _), found)
            | (expected, found @ CV::List(_, _))
            | (expected, found @ CV::Table(_, _)) => {
                return Err(anyhow!(
                    "failed to merge config value from `{}` into `{}`: expected {}, but found {}",
                    found.definition(),
                    expected.definition(),
                    expected.desc(),
                    found.desc()
                ));
            }
            (old, mut new) => {
                if force || new.definition().is_higher_priority(old.definition()) {
                    mem::swap(old, &mut new);
                }
            }
        }

        Ok(())
    }

    /// Extracts an integer and its definition location from a [`ConfigValue`].
    #[allow(dead_code)]
    pub(crate) fn i64(&self, key: &str) -> Result<(i64, &Definition)> {
        match self {
            CV::Integer(i, def) => Ok((*i, def)),
            _ => self.expected("integer", key),
        }
    }

    /// Extracts a string and its definition location from a [`ConfigValue`].
    #[allow(dead_code)]
    pub(crate) fn string(&self, key: &str) -> Result<(&str, &Definition)> {
        match self {
            CV::String(s, def) => Ok((s, def)),
            _ => self.expected("string", key),
        }
    }

    /// Extracts a table and its definition location from a [`ConfigValue`].
    #[allow(dead_code)]
    pub(crate) fn table(&self, key: &str) -> Result<(&HashMap<String, ConfigValue>, &Definition)> {
        match self {
            CV::Table(table, def) => Ok((table, def)),
            _ => self.expected("table", key),
        }
    }

    /// Extracts a list and its definition location from a [`ConfigValue`].
    #[allow(dead_code)]
    pub(crate) fn list(&self, key: &str) -> Result<&[(String, Definition)]> {
        match self {
            CV::List(list, _) => Ok(list),
            _ => self.expected("list", key),
        }
    }

    /// Extracts a boolean value and its definition location from a [`ConfigValue`].
    #[allow(dead_code)]
    pub(crate) fn boolean(&self, key: &str) -> Result<(bool, &Definition)> {
        match self {
            CV::Boolean(b, def) => Ok((*b, def)),
            _ => self.expected("bool", key),
        }
    }

    /// Returns a string description of the type of this [`ConfigValue`].
    pub(crate) fn desc(&self) -> &'static str {
        match *self {
            CV::Table(..) => "table",
            CV::List(..) => "array",
            CV::String(..) => "string",
            CV::Boolean(..) => "boolean",
            CV::Integer(..) => "integer",
        }
    }

    /// Extracts a [`Definition`] describing where this [`ConfigValue`] was defined.
    pub(crate) fn definition(&self) -> &Definition {
        match self {
            CV::Boolean(_, def)
            | CV::Integer(_, def)
            | CV::String(_, def)
            | CV::List(_, def)
            | CV::Table(_, def) => def,
        }
    }

    fn expected<T>(&self, wanted: &str, key: &str) -> Result<T> {
        bail!(
            "expected a {}, but found a {} for `{}` in {}",
            wanted,
            self.desc(),
            key,
            self.definition()
        )
    }
}

/// Returns the Substrate home directory.
pub(crate) fn homedir(cwd: &Path) -> Option<PathBuf> {
    crate::home::substrate_home_with_cwd(cwd).ok()
}

/// A type to deserialize a list of strings from a toml file.
///
/// Supports deserializing either a whitespace-separated list of arguments in a
/// single string or a string list itself. For example these deserialize to
/// equivalent values:
///
/// ```toml
/// a = 'a b c'
/// b = ['a', 'b', 'c']
/// ```
#[derive(Debug, Deserialize, Clone)]
pub(crate) struct StringList(Vec<String>);

impl StringList {
    /// Returns the [`StringList`] object as a [`String`] slice.
    #[allow(dead_code)]
    pub(crate) fn as_slice(&self) -> &[String] {
        &self.0
    }
}

/// StringList automatically merges config values with environment values,
/// this instead follows the precedence rules, so that eg. a string list found
/// in the environment will be used instead of one in a config file.
///
/// This is currently only used by `PathAndArgs`
#[derive(Debug, Deserialize)]
pub(crate) struct UnmergedStringList(Vec<String>);
