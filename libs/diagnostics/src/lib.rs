//! Utilities for collecting diagnostics.

#![warn(missing_docs)]

#[cfg(test)]
pub(crate) mod tests;

use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

/// A diagnostic issue that should be reported to users.
pub trait Diagnostic: Debug + Display {
    /// Returns an optional help message that should indicate
    /// what users need to do to resolve an issue.
    fn help(&self) -> Option<Box<dyn Display>> {
        None
    }

    /// Returns the severity of this issue.
    ///
    /// The default implementation returns [`Severity::default`].
    fn severity(&self) -> Severity {
        Default::default()
    }
}

/// An enumeration of possible severity levels.
#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Severity {
    /// An informational message.
    Info,
    /// A warning.
    #[default]
    Warning,
    /// An error. Often, but not always, fatal.
    Error,
}

/// A collection of issues.
#[derive(Debug, Clone)]
pub struct IssueSet<T> {
    issues: Vec<T>,
    num_errors: usize,
    num_warnings: usize,
}

impl<T> IssueSet<T> {
    /// Creates a new, empty issue set.
    #[inline]
    pub fn new() -> Self {
        Self {
            issues: Vec::new(),
            num_errors: 0,
            num_warnings: 0,
        }
    }

    /// Returns an iterator over all issues in the set.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.issues.iter()
    }

    /// The number of issues in this issue set.
    #[inline]
    pub fn len(&self) -> usize {
        self.issues.len()
    }

    /// Returns `true` if this issue set is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.issues.is_empty()
    }
}

impl<T: Diagnostic> IssueSet<T> {
    /// Adds the given issue to the issue set.
    #[inline]
    pub fn add(&mut self, issue: T) {
        let severity = issue.severity();
        match severity {
            Severity::Error => self.num_errors += 1,
            Severity::Warning => self.num_warnings += 1,
            _ => (),
        };
        self.issues.push(issue);
    }

    /// Returns `true` if this issue set contains an error.
    ///
    /// Errors are determined by [`Diagnostic`]s with a
    /// (severity)[Diagnostic::severity] of [`Severity::Error`].
    pub fn has_error(&self) -> bool {
        self.num_errors > 0
    }

    /// The number of errors in this issue set.
    #[inline]
    pub fn num_errors(&self) -> usize {
        self.num_errors
    }

    /// Returns `true` if this issue set contains a warning.
    ///
    /// Warnings are determined by [`Diagnostic`]s with a
    /// (severity)[Diagnostic::severity] of [`Severity::Warning`].
    pub fn has_warning(&self) -> bool {
        self.num_warnings > 0
    }

    /// The number of warnings in this issue set.
    #[inline]
    pub fn num_warnings(&self) -> usize {
        self.num_warnings
    }
}

impl<T> IntoIterator for IssueSet<T> {
    type Item = T;
    type IntoIter = <std::vec::Vec<T> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.issues.into_iter()
    }
}

impl<T> Default for IssueSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl Severity {
    /// Returns log level corresponding to this severity.
    #[inline]
    pub const fn as_tracing_level(&self) -> tracing::Level {
        match *self {
            Self::Info => tracing::Level::INFO,
            Self::Warning => tracing::Level::WARN,
            Self::Error => tracing::Level::ERROR,
        }
    }

    /// Returns `true` if the severity is [`Severity::Error`].
    #[inline]
    pub fn is_error(&self) -> bool {
        matches!(*self, Self::Error)
    }
}

impl Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
        }
    }
}

impl<T: Display> Display for IssueSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for issue in self.issues.iter() {
            writeln!(f, "{}", issue)?;
        }
        Ok(())
    }
}
