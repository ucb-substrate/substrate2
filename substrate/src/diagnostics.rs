// FIXME: unify crate with diagnostics crate?

use std::{borrow::Cow, panic::Location};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SourceInfo {
    // TODO: improve or replace with miette types
    file: Cow<'static, str>,
    line: u32,
    column: u32,
    length: Option<u32>,
}

impl SourceInfo {
    /// Get a [`SourceInfo`] pointing to the caller of `from_caller`
    ///
    /// Like with [`Location::caller`], annotate functions with
    /// `#[track_caller]` for `from_caller` to skip them when looking up the
    /// call stack
    #[track_caller]
    pub fn from_caller() -> Self {
        let loc = Location::caller();
        SourceInfo {
            file: loc.file().into(),
            line: loc.line(),
            column: loc.column(),
            length: None,
        }
    }
}
