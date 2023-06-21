use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

/// A diagnostic issue that should be reported to users.
pub trait Diagnostic: Debug + Display {
    fn help(&self) -> Option<String> {
        None
    }

    fn severity(&self) -> Severity {
        Default::default()
    }
}

/// An enumeration of possible severity levels.
#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    #[default]
    Warning,
    /// An error. Often, but not always, fatal.
    Error,
}

/// A collection of issues.
#[derive(Debug)]
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
