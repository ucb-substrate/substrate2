use std::collections::HashMap;
use std::fmt::Display;

use diagnostics::{IssueSet, Severity, Diagnostic};
use tracing::Level;

use super::*;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ValidatorIssue {
    cause: Cause,
    severity: Severity,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Cause {
    DuplicateCellNames {
        id1: CellId,
        id2: CellId,
        name: ArcStr,
    },
    DuplicateNodeNames {
        id1: NodeId,
        id2: NodeId,
        name: ArcStr,
        cell: CellId,
        cell_name: ArcStr,
    },
}

impl Diagnostic for ValidatorIssue {
    fn severity(&self) -> Severity {
        self.severity
    }
}

impl ValidatorIssue {
    pub fn new(cause: Cause, severity: Severity) -> Self {
        Self { cause, severity }
    }

    pub(crate) fn new_and_log(cause: Cause, severity: Severity) -> Self {
        let result = Self::new(cause, severity);
        match severity {
            Severity::Info => tracing::event!(Level::INFO, issue = ?result.cause, "{}", result),
            Severity::Warning => tracing::event!(Level::WARN, issue = ?result.cause, "{}", result),
            Severity::Error => tracing::event!(Level::ERROR, issue = ?result.cause, "{}", result),
        }
        result
    }
}

impl Display for ValidatorIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cause)
    }
}

impl Display for Cause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateCellNames { id1, id2, name } => write!(
                f,
                "duplicate cell names: found two or more cells named `{}`",
                name
            ),
            Self::DuplicateNodeNames {
                id1,
                id2,
                name,
                cell,
                cell_name,
            } => write!(
                f,
                "duplicate node names: found two or more nodes named `{}` in cell `{}`",
                name, cell_name
            ),
        }
    }
}

impl Library {
    pub fn validate(&self) -> IssueSet<ValidatorIssue> {
        let _guard = span!(Level::INFO, "validating SCIR Library").entered();
        let mut issues = IssueSet::new();

        let mut cell_names = HashMap::new();
        for (id, cell) in self.cells.iter() {
            self.validate_cell(*id, &mut issues);
            if let Some(id1) = cell_names.insert(cell.name.clone(), id) {
                let issue = ValidatorIssue {
                        cause: Cause::DuplicateCellNames {
                            id1: *id1,
                            id2: *id,
                            name: cell.name.clone(),
                        },
                        severity: Severity::Warning,
                    };

                issues.add(issue);
            }
        }

        issues
    }

    fn validate_cell(&self, cell: CellId, issues: &mut IssueSet<ValidatorIssue>) {}
}

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn duplicate_cell_names() {
        let c1 = Cell::new("duplicate_cell_name");
        let c2 = Cell::new("duplicate_cell_name");
        let mut lib = Library::new();
        lib.add_cell(c1);
        lib.add_cell(c2);
    }
}
