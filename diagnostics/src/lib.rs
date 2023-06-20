use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

pub trait Diagnostic: Debug + Display {
    fn help(&self) -> Option<String> {
        None
    }

    fn severity(&self) -> Severity {
        Default::default()
    }
}

#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    #[default]
    Warning,
    Error,
}

#[derive(Debug)]
pub struct IssueSet<T> {
    issues: Vec<T>,
}

impl<T> IssueSet<T> {
    #[inline]
    pub fn new() -> Self {
        Self { issues: Vec::new() }
    }

    #[inline]
    pub fn add(&mut self, issue: T) {
        self.issues.push(issue);
    }

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
    #[inline]
    pub const fn as_tracing_level(&self) -> tracing::Level {
        match *self {
            Self::Info => tracing::Level::INFO,
            Self::Warning => tracing::Level::WARN,
            Self::Error => tracing::Level::ERROR,
        }
    }
}
