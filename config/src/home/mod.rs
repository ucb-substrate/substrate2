//! Canonical definitions of [`home_dir`] and [`substrate_home`].
//!
//! The definition of [`home_dir`] provided by the standard library is
//! incorrect because it considers the `HOME` environment variable on
//! Windows. This causes surprising situations where a Rust program
//! will behave differently depending on whether it is run under a
//! Unix emulation environment like Cygwin or MinGW. Substrate does not
//! use the standard libraries definition and instead uses the
//! definition here.
//!
//! This crate provides an additional function, [`substrate_home`],
//! which is the canonical way to determine the
//! location that Substrate stores its data.
//! The [`mod@env`] module contains utilities for mocking the process environment
//! by Substrate.
//!
//
// ## LICENSING
//
// Based on Cargo's [`home` crate](https://github.com/rust-lang/cargo/tree/master/crates/home)
// with substantial modifications.

pub(crate) mod env;

#[cfg(target_os = "windows")]
mod windows;

use std::io;
use std::path::{Path, PathBuf};

/// Returns the path of the current user's home directory using environment
/// variables or OS-specific APIs.
///
/// # Unix
///
/// Returns the value of the `HOME` environment variable if it is set
/// **even** if it is an empty string. Otherwise, it tries to determine the
/// home directory by invoking the [`getpwuid_r`][getpwuid] function with
/// the UID of the current user.
///
/// [getpwuid]: https://linux.die.net/man/3/getpwuid_r
///
/// # Windows
///
/// Returns the value of the `USERPROFILE` environment variable if it is set
/// **and** it is not an empty string. Otherwise, it tries to determine the
/// home directory by invoking the [`SHGetFolderPathW`][shgfp] function with
/// [`CSIDL_PROFILE`][csidl].
///
/// [shgfp]: https://docs.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shgetfolderpathw
/// [csidl]: https://learn.microsoft.com/en-us/windows/win32/shell/csidl
///
/// # Examples
///
/// ```rust,ignore
/// match config::home::home_dir() {
///     Some(path) if !path.as_os_str().is_empty() => println!("{}", path.display()),
///     _ => println!("Unable to get your home dir!"),
/// }
/// ```
#[allow(dead_code)]
pub(crate) fn home_dir() -> Option<PathBuf> {
    env::home_dir_with_env(&env::OS_ENV)
}

#[cfg(windows)]
use windows::home_dir_inner;

#[cfg(any(unix, target_os = "redox"))]
fn home_dir_inner() -> Option<PathBuf> {
    #[allow(deprecated)]
    std::env::home_dir()
}

/// Returns the storage directory used by Substrate, often known as
/// `.substrate` or `SUBSTRATE_HOME`.
///
/// It returns one of the following values, in this order of
/// preference:
///
/// - The value of the `SUBSTRATE_HOME` environment variable, if it is
///   an absolute path.
/// - The value of the current working directory joined with the value
///   of the `SUBSTRATE_HOME` environment variable, if `SUBSTRATE_HOME` is a
///   relative directory.
/// - The `.substrate` directory in the user's home directory, as reported
///   by the `home_dir` function.
///
/// # Errors
///
/// This function fails if it fails to retrieve the current directory,
/// or if the home directory cannot be determined.
///
/// # Examples
///
/// ```rust,ignore
/// match config::home::substrate_home() {
///     Ok(path) => println!("{}", path.display()),
///     Err(err) => eprintln!("Cannot get your Substrate home dir: {:?}", err),
/// }
/// ```
#[allow(dead_code)]
pub(crate) fn substrate_home() -> io::Result<PathBuf> {
    env::substrate_home_with_env(&env::OS_ENV)
}

/// Returns the storage directory used by Substrate within `cwd`.
/// For more details, see [`substrate_home`].
#[allow(dead_code)]
pub(crate) fn substrate_home_with_cwd(cwd: &Path) -> io::Result<PathBuf> {
    env::substrate_home_with_cwd_env(&env::OS_ENV, cwd)
}
