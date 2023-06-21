//! SCIR validation utilities.
//!
//! This module provides helpers for ensuring that SCIR libraries
//! and cells are valid.

use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use diagnostics::{Diagnostic, IssueSet, Severity};

use super::*;

/// An issue identified during validation of an SCIR library.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ValidatorIssue {
    cause: Cause,
    severity: Severity,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Cause {
    /// Two cells have the same name.
    DuplicateCellNames {
        id1: CellId,
        id2: CellId,
        name: ArcStr,
    },
    /// Two nodes in a cell have the same name.
    DuplicateNodeNames {
        id1: NodeId,
        id2: NodeId,
        name: ArcStr,
        cell_id: CellId,
        cell_name: ArcStr,
    },
    /// A node is listed as a port more than once.
    ShortedPorts {
        node: NodeId,
        name: ArcStr,
        cell_id: CellId,
        cell_name: ArcStr,
    },
    /// A node identifier is used but not declared.
    MissingNode {
        id: NodeId,
        cell_id: CellId,
        cell_name: ArcStr,
    },
    /// An instance in a parent cell references a child cell not present in the library.
    MissingChildCell {
        child_cell_id: CellId,
        parent_cell_id: CellId,
        parent_cell_name: ArcStr,
        instance_name: ArcStr,
    },
    /// An instance does not specify a connection to a port of its child cell.
    UnconnectedPort {
        child_cell_id: CellId,
        child_cell_name: ArcStr,
        port: ArcStr,
        parent_cell_id: CellId,
        parent_cell_name: ArcStr,
        instance_name: ArcStr,
    },
    /// An instance specifies a connection to a port that does not exist in the child cell.
    ExtraPort {
        child_cell_id: CellId,
        child_cell_name: ArcStr,
        port: ArcStr,
        parent_cell_id: CellId,
        parent_cell_name: ArcStr,
        instance_name: ArcStr,
    },
    /// An instance does not specify a parameter value, and the parameter has no default.
    MissingParam {
        child_cell_id: CellId,
        child_cell_name: ArcStr,
        param: ArcStr,
        parent_cell_id: CellId,
        parent_cell_name: ArcStr,
        instance_name: ArcStr,
    },
    /// An instance specifies a parameter value, but the parameter does not exist.
    ExtraParam {
        child_cell_id: CellId,
        child_cell_name: ArcStr,
        param: ArcStr,
        parent_cell_id: CellId,
        parent_cell_name: ArcStr,
        instance_name: ArcStr,
    },
}

impl Diagnostic for ValidatorIssue {
    fn severity(&self) -> Severity {
        self.severity
    }
}

impl ValidatorIssue {
    /// Creates a new validator issue from the given cause and severity.
    pub(crate) fn new(cause: Cause, severity: Severity) -> Self {
        Self { cause, severity }
    }

    /// Gets the underlying cause of this issue.
    #[inline]
    pub fn cause(&self) -> &Cause {
        &self.cause
    }

    /// Creates a new validator issue and logs it immediately.
    ///
    /// The log level will be selected according to the given severity.
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
            Self::DuplicateCellNames { name, .. } => write!(
                f,
                "duplicate cell names: found two or more cells named `{}`",
                name
            ),
            Self::DuplicateNodeNames {
                name, cell_name, ..
            } => write!(
                f,
                "duplicate node names: found two or more nodes named `{}` in cell `{}`",
                name, cell_name
            ),
            Self::ShortedPorts { name, cell_name, .. } =>
                write!(f, "shorted ports: port `{}` in cell `{}` is connected to a node already used by another port", name, cell_name),

            Self::MissingNode { id, cell_name, .. } =>
                write!(f, "invalid node ID {} in cell `{}`", id, cell_name),

            Self::MissingChildCell { child_cell_id, parent_cell_name, instance_name, .. } =>
                write!(f, "missing child cell: instance `{}` in cell `{}` references cell ID `{}`, but no cell with this ID was found in the library", instance_name, parent_cell_name, child_cell_id),

            Self::UnconnectedPort { child_cell_name, port, parent_cell_name, instance_name, .. } =>
                write!(f, "unconnected port: instance `{}` in cell `{}` does not specify a connection for port `{}` of cell `{}`", instance_name, parent_cell_name, port, child_cell_name),

            Self::ExtraPort { child_cell_name, port, parent_cell_name, instance_name, .. } =>
                write!(f, "extra port: instance `{}` in cell `{}` specifies a connection for port `{}` of cell `{}`, but this cell has no such port", instance_name, parent_cell_name, port, child_cell_name),

            Self::MissingParam { child_cell_name, param, parent_cell_name, instance_name, .. } =>
                write!(f, "unspecified parameter: instance `{}` in cell `{}` does not specify a value for parameter `{}` of cell `{}`, and this parameter does not have a default value", instance_name, parent_cell_name, param, child_cell_name),

            Self::ExtraParam { child_cell_name, param, parent_cell_name, instance_name, .. } =>
                write!(f, "extra param: instance `{}` in cell `{}` specifies a value for parameter `{}` of cell `{}`, but this cell has no such parameter", instance_name, parent_cell_name, param, child_cell_name),
        }
    }
}

impl Library {
    /// Check whether or not this library is valid.
    pub fn validate(&self) -> IssueSet<ValidatorIssue> {
        let _guard = span!(Level::INFO, "validating SCIR Library").entered();
        let mut issues = IssueSet::new();
        self.validate1(&mut issues);

        if issues.has_error() {
            return issues;
        }

        self.validate2(&mut issues);
        issues
    }

    fn validate1(&self, issues: &mut IssueSet<ValidatorIssue>) {
        let _guard = span!(
            Level::INFO,
            "validation pass 1 (checking node and port identifier validity)"
        )
        .entered();

        let mut cell_names = HashMap::new();
        for (id, cell) in self.cells.iter() {
            self.validate_cell1(*id, issues);
            if let Some(id1) = cell_names.insert(cell.name.clone(), id) {
                let issue = ValidatorIssue::new_and_log(
                    Cause::DuplicateCellNames {
                        id1: *id1,
                        id2: *id,
                        name: cell.name.clone(),
                    },
                    Severity::Warning,
                );
                issues.add(issue);
            }
        }
    }

    fn validate2(&self, issues: &mut IssueSet<ValidatorIssue>) {
        let _guard = span!(
            Level::INFO,
            "validation pass 2 (checking connection validity)"
        )
        .entered();
        for id in self.cells.keys().copied() {
            self.validate_cell2(id, issues);
        }
    }

    fn validate_cell1(&self, id: CellId, issues: &mut IssueSet<ValidatorIssue>) {
        let cell = self.cells.get(&id).unwrap();
        let _guard =
            span!(Level::INFO, "validating SCIR cell (pass 1)", cell.id = %id, cell.name = %cell.name)
                .entered();

        let invalid_node = |node_id: NodeId| {
            ValidatorIssue::new_and_log(
                Cause::MissingNode {
                    id: node_id,
                    cell_id: id,
                    cell_name: cell.name.clone(),
                },
                Severity::Error,
            )
        };

        for instance in cell.instances.iter() {
            for node in instance.connections.values().copied() {
                if !cell.nodes.contains_key(&node) {
                    issues.add(invalid_node(node));
                }
            }
        }

        for device in cell.primitives.iter() {
            for node in device.nodes() {
                if !cell.nodes.contains_key(&node) {
                    issues.add(invalid_node(node));
                }
            }
        }

        let mut port_nodes = HashSet::with_capacity(cell.ports.len());
        for port in cell.ports.iter() {
            if !cell.nodes.contains_key(&port.node) {
                issues.add(invalid_node(port.node));
                continue;
            }

            if !port_nodes.insert(port.node) {
                let issue = ValidatorIssue::new_and_log(
                    Cause::ShortedPorts {
                        node: port.node,
                        name: cell.nodes.get(&port.node).unwrap().name.clone(),
                        cell_id: id,
                        cell_name: cell.name.clone(),
                    },
                    Severity::Error,
                );
                issues.add(issue);
            }
        }
    }

    fn validate_cell2(&self, id: CellId, issues: &mut IssueSet<ValidatorIssue>) {
        let cell = self.cells.get(&id).unwrap();
        let _guard =
            span!(Level::INFO, "validating SCIR cell (pass 2)", cell.id = %id, cell.name = %cell.name)
                .entered();

        for instance in cell.instances.iter() {
            let child = match self.cells.get(&instance.cell) {
                Some(child) => child,
                None => {
                    let issue = ValidatorIssue::new_and_log(
                        Cause::MissingChildCell {
                            child_cell_id: instance.cell,
                            parent_cell_id: id,
                            parent_cell_name: cell.name.clone(),
                            instance_name: instance.name.clone(),
                        },
                        Severity::Error,
                    );
                    issues.add(issue);
                    continue;
                }
            };

            let mut child_ports = HashSet::with_capacity(child.ports.len());

            // Check for missing ports
            for port in child.ports.iter() {
                let name = &child.nodes[&port.node].name;
                child_ports.insert(name.clone());
                if !instance.connections.contains_key(name) {
                    let issue = ValidatorIssue::new_and_log(
                        Cause::UnconnectedPort {
                            child_cell_id: instance.cell,
                            child_cell_name: child.name.clone(),
                            port: name.clone(),
                            parent_cell_name: cell.name.clone(),
                            parent_cell_id: id,
                            instance_name: instance.name.clone(),
                        },
                        Severity::Error,
                    );
                    issues.add(issue);
                }
            }

            // Check for extra ports
            for conn in instance.connections.keys() {
                if !child_ports.contains(conn) {
                    let issue = ValidatorIssue::new_and_log(
                        Cause::ExtraPort {
                            child_cell_id: instance.cell,
                            child_cell_name: child.name.clone(),
                            port: conn.clone(),
                            parent_cell_name: cell.name.clone(),
                            parent_cell_id: id,
                            instance_name: instance.name.clone(),
                        },
                        Severity::Error,
                    );
                    issues.add(issue);
                }
            }

            // Check for missing params.
            let mut child_params = HashSet::with_capacity(child.params.len());
            for (name, param) in child.params.iter() {
                child_params.insert(name.clone());
                if instance.params.get(name).is_none() && !param.has_default() {
                    let issue = ValidatorIssue::new_and_log(
                        Cause::MissingParam {
                            child_cell_id: instance.cell,
                            child_cell_name: child.name.clone(),
                            param: name.clone(),
                            parent_cell_name: cell.name.clone(),
                            parent_cell_id: id,
                            instance_name: instance.name.clone(),
                        },
                        Severity::Error,
                    );
                    issues.add(issue);
                }
            }

            // Check for extra params.
            for param in instance.params.keys() {
                if !child_params.contains(param) {
                    let issue = ValidatorIssue::new_and_log(
                        Cause::ExtraParam {
                            child_cell_id: instance.cell,
                            child_cell_name: child.name.clone(),
                            param: param.clone(),
                            parent_cell_name: cell.name.clone(),
                            parent_cell_id: id,
                            instance_name: instance.name.clone(),
                        },
                        Severity::Warning,
                    );
                    issues.add(issue);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use test_log::test;

    #[test]
    fn duplicate_cell_names() {
        let c1 = Cell::new("duplicate_cell_name");
        let c2 = Cell::new("duplicate_cell_name");
        let mut lib = Library::new();
        lib.add_cell(c1);
        lib.add_cell(c2);
        let issues = lib.validate();
        assert!(issues.has_error() || issues.has_warning());
    }
}
