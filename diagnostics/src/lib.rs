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
}

impl<T> IssueSet<T> {
    /// Creates a new, empty issue set.
    #[inline]
    pub fn new() -> Self {
        Self { issues: Vec::new() }
    }

    /// Adds the given issue to the issue set.
    #[inline]
    pub fn add(&mut self, issue: T) {
        self.issues.push(issue);
    }

    /// Returns an iterator over all issues in the set.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.issues.iter()
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
}
