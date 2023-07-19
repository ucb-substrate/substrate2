//! Configuration path utilities.
//
// ## LICENSING
//
// Based on Cargo's [`config` module](https://github.com/rust-lang/cargo/tree/master/src/cargo/util/config)
// with substantial modifications.

use super::{RawConfig, UnmergedStringList, Value};
use serde::{de::Error, Deserialize};
use std::path::PathBuf;

/// Use with the `get` API to fetch a string that will be converted to a
/// `PathBuf`. Relative paths are converted to absolute paths based on the
/// location of the config file.
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(transparent)]
pub(crate) struct ConfigRelativePath(Value<String>);

impl ConfigRelativePath {
    /// Returns the underlying value.
    #[allow(dead_code)]
    pub(crate) fn value(&self) -> &Value<String> {
        &self.0
    }

    /// Returns the raw underlying configuration value for this key.
    #[allow(dead_code)]
    pub(crate) fn raw_value(&self) -> &str {
        &self.0.val
    }

    /// Resolves this configuration-relative path to an absolute path.
    ///
    /// This will always return an absolute path where it's relative to the
    /// location for configuration for this value.
    #[allow(dead_code)]
    pub(crate) fn resolve_path(&self, config: &RawConfig) -> PathBuf {
        self.0.definition.root(config).join(&self.0.val)
    }

    /// Resolves this configuration-relative path to either an absolute path or
    /// something appropriate to execute from `PATH`.
    ///
    /// Values which don't look like a filesystem path (don't contain `/` or
    /// `\`) will be returned as-is, and everything else will fall through to an
    /// absolute path.
    pub(crate) fn resolve_program(&self, config: &RawConfig) -> PathBuf {
        config.string_to_path(&self.0.val, &self.0.definition)
    }
}

/// A config type that is a program to run.
///
/// This supports a list of strings like `['/path/to/program', 'somearg']`
/// or a space separated string like `'/path/to/program somearg'`.
///
/// This expects the first value to be the path to the program to run.
/// Subsequent values are strings of arguments to pass to the program.
///
/// Typically you should use `ConfigRelativePath::resolve_program` on the path
/// to get the actual program.
#[derive(Debug, Clone)]
pub(crate) struct PathAndArgs {
    #[allow(dead_code)]
    pub(crate) path: ConfigRelativePath,
    #[allow(dead_code)]
    pub(crate) args: Vec<String>,
}

impl<'de> serde::Deserialize<'de> for PathAndArgs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let vsl = Value::<UnmergedStringList>::deserialize(deserializer)?;
        let mut strings = vsl.val.0;
        if strings.is_empty() {
            return Err(D::Error::invalid_length(0, &"at least one element"));
        }
        let first = strings.remove(0);
        let crp = Value {
            val: first,
            definition: vsl.definition,
        };
        Ok(PathAndArgs {
            path: ConfigRelativePath(crp),
            args: strings,
        })
    }
}
