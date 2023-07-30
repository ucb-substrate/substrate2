//! Various utilities for working with files and paths.
//
// ## LICENSING
//
// Based on Cargo's utility functions with substantial modifications.

use anyhow::{Context, Result};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Returns an iterator that walks up the directory hierarchy towards the root.
///
/// Each item is a [`Path`]. It will start with the given path, finishing at
/// the root. If the `stop_root_at` parameter is given, it will stop at the
/// given path (which will be the last item).
pub(crate) fn ancestors<'a>(path: &'a Path, stop_root_at: Option<&Path>) -> PathAncestors<'a> {
    PathAncestors::new(path, stop_root_at)
}

/// An iterator over parent paths from the current directory to a certain stopping directory.
pub(crate) struct PathAncestors<'a> {
    current: Option<&'a Path>,
    stop_at: Option<PathBuf>,
}

impl<'a> PathAncestors<'a> {
    fn new(path: &'a Path, stop_root_at: Option<&Path>) -> PathAncestors<'a> {
        let stop_at = stop_root_at.map(|p| p.to_path_buf());
        PathAncestors {
            current: Some(path),
            stop_at,
        }
    }
}

impl<'a> Iterator for PathAncestors<'a> {
    type Item = &'a Path;

    fn next(&mut self) -> Option<&'a Path> {
        if let Some(path) = self.current {
            self.current = path.parent();

            if let Some(ref stop_at) = self.stop_at {
                if path == stop_at {
                    self.current = None;
                }
            }

            Some(path)
        } else {
            None
        }
    }
}

/// Equivalent to [`std::fs::create_dir_all`] with better error messages.
#[allow(dead_code)]
pub(crate) fn create_dir_all(p: impl AsRef<Path>) -> Result<()> {
    _create_dir_all(p.as_ref())
}

fn _create_dir_all(p: &Path) -> Result<()> {
    fs::create_dir_all(p)
        .with_context(|| format!("failed to create directory `{}`", p.display()))?;
    Ok(())
}

/// Equivalent to [`std::fs::remove_dir_all`] with better error messages.
///
/// This does *not* follow symlinks.
#[allow(dead_code)]
pub(crate) fn remove_dir_all<P: AsRef<Path>>(p: P) -> Result<()> {
    _remove_dir_all(p.as_ref()).or_else(|prev_err| {
        // `std::fs::remove_dir_all` is highly specialized for different platforms
        // and may be more reliable than a simple walk. We try the walk first in
        // order to report more detailed errors.
        fs::remove_dir_all(p.as_ref()).with_context(|| {
            format!(
                "{:?}\n\nError: failed to remove directory `{}`",
                prev_err,
                p.as_ref().display(),
            )
        })
    })
}

fn _remove_dir_all(p: &Path) -> Result<()> {
    if p.symlink_metadata()
        .with_context(|| format!("could not get metadata for `{}` to remove", p.display()))?
        .is_symlink()
    {
        return remove_file(p);
    }
    let entries = p
        .read_dir()
        .with_context(|| format!("failed to read directory `{}`", p.display()))?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            remove_dir_all(&path)?;
        } else {
            remove_file(&path)?;
        }
    }
    remove_dir(p)
}

/// Equivalent to [`std::fs::remove_dir`] with better error messages.
#[allow(dead_code)]
pub(crate) fn remove_dir<P: AsRef<Path>>(p: P) -> Result<()> {
    _remove_dir(p.as_ref())
}

fn _remove_dir(p: &Path) -> Result<()> {
    fs::remove_dir(p).with_context(|| format!("failed to remove directory `{}`", p.display()))?;
    Ok(())
}

/// Equivalent to [`std::fs::remove_file`] with better error messages.
///
/// If the file is readonly, this will attempt to change the permissions to
/// force the file to be deleted.
pub(crate) fn remove_file<P: AsRef<Path>>(p: P) -> Result<()> {
    _remove_file(p.as_ref())
}

fn _remove_file(p: &Path) -> Result<()> {
    let mut err = match fs::remove_file(p) {
        Ok(()) => return Ok(()),
        Err(e) => e,
    };

    if err.kind() == io::ErrorKind::PermissionDenied && set_not_readonly(p).unwrap_or(false) {
        match fs::remove_file(p) {
            Ok(()) => return Ok(()),
            Err(e) => err = e,
        }
    }

    Err(err).with_context(|| format!("failed to remove file `{}`", p.display()))?;
    Ok(())
}

#[allow(clippy::permissions_set_readonly_false)]
fn set_not_readonly(p: &Path) -> io::Result<bool> {
    let mut perms = p.metadata()?.permissions();
    if !perms.readonly() {
        return Ok(false);
    }
    perms.set_readonly(false);
    fs::set_permissions(p, perms)?;
    Ok(true)
}
