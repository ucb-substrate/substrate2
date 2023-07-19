//! Lower-level utilities for mocking the process environment.
//
// ## LICENSING
//
// Based on Cargo's [`home` crate](https://github.com/rust-lang/cargo/tree/master/crates/home)
// with substantial modifications.

use std::{
    ffi::OsString,
    io,
    path::{Path, PathBuf},
};

/// Permits parameterizing the home functions via the _from variants.
pub(crate) trait Env {
    /// Return the path to the users home dir, or None if any error occurs:
    /// see home_inner.
    fn home_dir(&self) -> Option<PathBuf>;
    /// Return the current working directory.
    fn current_dir(&self) -> io::Result<PathBuf>;
    /// Get an environment variable, as per std::env::var_os.
    fn var_os(&self, key: &str) -> Option<OsString>;
}

/// Implements Env for the OS context, both Unix style and Windows.
///
/// This trait permits in-process testing by providing a control point to
/// allow in-process divergence on what is normally process wide state.
///
/// Implementations should be provided by whatever testing framework the caller
/// is using.
pub(crate) struct OsEnv;
impl Env for OsEnv {
    fn home_dir(&self) -> Option<PathBuf> {
        crate::home::home_dir_inner()
    }
    fn current_dir(&self) -> io::Result<PathBuf> {
        std::env::current_dir()
    }
    fn var_os(&self, key: &str) -> Option<OsString> {
        std::env::var_os(key)
    }
}

/// The current OS context.
pub(crate) const OS_ENV: OsEnv = OsEnv;

/// Returns the path of the current user's home directory from [`Env::home_dir`].
pub(crate) fn home_dir_with_env(env: &dyn Env) -> Option<PathBuf> {
    env.home_dir()
}

/// Variant of substrate_home where the environment source is parameterized. This is
/// specifically to support in-process testing scenarios as environment
/// variables and user home metadata are normally process global state. See the
/// [`Env`] trait.
pub(crate) fn substrate_home_with_env(env: &dyn Env) -> io::Result<PathBuf> {
    let cwd = env.current_dir()?;
    substrate_home_with_cwd_env(env, &cwd)
}

/// Variant of substrate_home_with_cwd where the environment source is
/// parameterized. This is specifically to support in-process testing scenarios
/// as environment variables and user home metadata are normally process global
/// state. See the [`OsEnv`] trait.
pub(crate) fn substrate_home_with_cwd_env(env: &dyn Env, cwd: &Path) -> io::Result<PathBuf> {
    match env.var_os("SUBSTRATE_HOME").filter(|h| !h.is_empty()) {
        Some(home) => {
            let home = PathBuf::from(home);
            if home.is_absolute() {
                Ok(home)
            } else {
                Ok(cwd.join(&home))
            }
        }
        _ => home_dir_with_env(env)
            .map(|p| p.join(".substrate"))
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::Other, "could not find Substrate home dir")
            }),
    }
}
