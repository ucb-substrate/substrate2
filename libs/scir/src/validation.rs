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
    /// Two instances in the same cell have the same name.
    DuplicateInstanceNames {
        inst_name: ArcStr,
        cell_id: CellId,
        cell_name: ArcStr,
    },
    /// Two signals in a cell have the same name.
    DuplicateSignalNames {
        id1: SignalId,
        id2: SignalId,
        name: ArcStr,
        cell_id: CellId,
        cell_name: ArcStr,
    },
    /// A signal is listed as a port more than once.
    ShortedPorts {
        signal: SignalId,
        name: ArcStr,
        cell_id: CellId,
        cell_name: ArcStr,
    },
    /// A signal identifier is used but not declared.
    MissingSignal {
        id: SignalId,
        cell_id: CellId,
        cell_name: ArcStr,
    },
    /// An instance in a parent cell references a child not present in the library.
    MissingChild {
        child_id: ChildId,
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
    /// A bus index is out of bounds given the width of the bus.
    IndexOutOfBounds {
        idx: usize,
        width: usize,
        cell_id: CellId,
        cell_name: ArcStr,
    },
    /// Used a bus without indexing into it.
    MissingIndex {
        signal_name: ArcStr,
        cell_id: CellId,
        cell_name: ArcStr,
    },
    /// Attempted to index a single wire.
    IndexedWire {
        signal_name: ArcStr,
        cell_id: CellId,
        cell_name: ArcStr,
    },
    /// An instance specified a connection of incorrect width.
    PortWidthMismatch {
        expected_width: usize,
        actual_width: usize,
        instance_name: ArcStr,
        port: ArcStr,
        parent_cell_id: CellId,
        parent_cell_name: ArcStr,
        child_cell_id: CellId,
        child_cell_name: ArcStr,
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
            Self::DuplicateInstanceNames { inst_name, cell_name, .. } => write!(
                f,
                "duplicate instance names: found two or more instances named `{}` in cell `{}`",
                inst_name, cell_name,
            ),
            Self::DuplicateSignalNames {
                name, cell_name, ..
            } => write!(
                f,
                "duplicate signal names: found two or more signals named `{}` in cell `{}`",
                name, cell_name
            ),
            Self::ShortedPorts { name, cell_name, .. } =>
                write!(
                    f,
                    "shorted ports: port `{}` in cell `{}` is connected to a signal already used by another port",
                    name,
                    cell_name
                ),

            Self::MissingSignal { id, cell_name, .. } =>
                write!(
                    f,
                    "invalid signal ID {} in cell `{}`",
                    id,
                    cell_name
                ),

            Self::MissingChild { child_id, parent_cell_name, instance_name, .. } =>
                write!(
                    f,
                    "missing child cell: instance `{}` in cell `{}` references cell ID `{}`, but no cell with this ID was found in the library",
                    instance_name,
                    parent_cell_name,
                    child_id
                ),

            Self::UnconnectedPort { child_cell_name, port, parent_cell_name, instance_name, .. } =>
                write!(
                    f,
                    "unconnected port: instance `{}` in cell `{}` does not specify a connection for port `{}` of cell `{}`",
                    instance_name,
                    parent_cell_name,
                    port,
                    child_cell_name
                ),

            Self::ExtraPort { child_cell_name, port, parent_cell_name, instance_name, .. } =>
                write!(
                    f,
                    "extra port: instance `{}` in cell `{}` specifies a connection for port `{}` of cell `{}`, but this cell has no such port",
                    instance_name,
                    parent_cell_name,
                    port,
                    child_cell_name
                ),

            Self::IndexOutOfBounds {idx, width, cell_name, .. } =>
                write!(
                    f,
                    "index out of bounds: attempted to access index {} of signal with width {} in cell `{}`",
                    idx,
                    width,
                    cell_name
                ),

            Self::MissingIndex { signal_name, cell_name, .. } =>
                write!(
                    f,
                    "missing index on use of bus signal `{}` in cell `{}`",
                    signal_name,
                    cell_name
                ),

            Self::IndexedWire { signal_name, cell_name, .. } =>
                write!(
                    f,
                    "attempted to index a single-bit wire: signal `{}` in cell `{}`",
                    signal_name,
                    cell_name
                ),

            Self::PortWidthMismatch { expected_width, actual_width, instance_name, port, parent_cell_name, child_cell_name, .. } =>
                write!(
                    f,
                    "mismatched port width: instance `{}` in cell `{}` specifies a connection to port `{}` of cell `{}` of width {}, but the expected width is {}",
                    instance_name,
                    parent_cell_name,
                    port,
                    child_cell_name,
                    actual_width,
                    expected_width
                ),

        }
    }
}

impl<S: Schema + ?Sized> LibraryBuilder<S> {
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
            "validation pass 1 (checking signal and port identifier validity)"
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
                    Severity::Error,
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

        let invalid_signal = |signal_id: SignalId| {
            ValidatorIssue::new_and_log(
                Cause::MissingSignal {
                    id: signal_id,
                    cell_id: id,
                    cell_name: cell.name.clone(),
                },
                Severity::Error,
            )
        };

        let mut inst_names = HashSet::new();
        for (_id, instance) in cell.instances.iter() {
            if inst_names.contains(&instance.name) {
                issues.add(ValidatorIssue::new_and_log(
                    Cause::DuplicateInstanceNames {
                        inst_name: instance.name.clone(),
                        cell_id: id,
                        cell_name: cell.name.clone(),
                    },
                    Severity::Error,
                ));
            }
            inst_names.insert(instance.name.clone());
            for concat in instance.connections.values() {
                for part in concat.parts() {
                    let signal = match cell.signals.get(&part.signal()) {
                        Some(signal) => signal,
                        None => {
                            issues.add(invalid_signal(part.signal()));
                            continue;
                        }
                    };

                    // check out of bounds indexing.
                    match (signal.width, part.range()) {
                        (Some(width), Some(range)) => {
                            if range.end > width {
                                issues.add(ValidatorIssue::new_and_log(
                                    Cause::IndexOutOfBounds {
                                        idx: range.end,
                                        width,
                                        cell_id: id,
                                        cell_name: cell.name.clone(),
                                    },
                                    Severity::Error,
                                ));
                            }
                        }
                        (Some(_), None) => {
                            issues.add(ValidatorIssue::new_and_log(
                                Cause::MissingIndex {
                                    signal_name: signal.name.clone(),
                                    cell_id: id,
                                    cell_name: cell.name.clone(),
                                },
                                Severity::Error,
                            ));
                        }
                        (None, Some(_)) => {
                            issues.add(ValidatorIssue::new_and_log(
                                Cause::IndexedWire {
                                    signal_name: signal.name.clone(),
                                    cell_id: id,
                                    cell_name: cell.name.clone(),
                                },
                                Severity::Error,
                            ));
                        }
                        (None, None) => {}
                    }
                }
            }
        }

        let mut port_signals = HashSet::with_capacity(cell.ports.len());
        for port in cell.ports() {
            if !cell.signals.contains_key(&port.signal) {
                issues.add(invalid_signal(port.signal));
                continue;
            }

            if !port_signals.insert(port.signal) {
                let issue = ValidatorIssue::new_and_log(
                    Cause::ShortedPorts {
                        signal: port.signal,
                        name: cell.signals.get(&port.signal).unwrap().name.clone(),
                        cell_id: id,
                        cell_name: cell.name.clone(),
                    },
                    Severity::Error,
                );
                issues.add(issue);
            }
        }

        let mut signal_names = HashMap::new();
        for (signal_id, signal) in cell.signals() {
            if let Some(other) = signal_names.insert(&signal.name, signal_id) {
                let issue = ValidatorIssue::new_and_log(
                    Cause::DuplicateSignalNames {
                        id1: signal_id,
                        id2: other,
                        name: signal.name.clone(),
                        cell_id: id,
                        cell_name: cell.name().clone(),
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

        for (_id, instance) in cell.instances.iter() {
            match instance.child {
                ChildId::Cell(c) => {
                    let child = match self.cells.get(&c) {
                        Some(child) => child,
                        None => {
                            let issue = ValidatorIssue::new_and_log(
                                Cause::MissingChild {
                                    child_id: c.into(),
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
                    for port in child.ports() {
                        let name = &child.signals[&port.signal].name;
                        child_ports.insert(name.clone());
                        match instance.connections.get(name) {
                            Some(conn) => {
                                let expected_width = child.signals[&port.signal].width.unwrap_or(1);
                                if conn.width() != expected_width {
                                    let issue = ValidatorIssue::new_and_log(
                                        Cause::PortWidthMismatch {
                                            expected_width,
                                            actual_width: conn.width(),
                                            port: name.clone(),
                                            instance_name: instance.name.clone(),
                                            child_cell_id: instance.child.unwrap_cell(),
                                            child_cell_name: child.name.clone(),
                                            parent_cell_name: cell.name.clone(),
                                            parent_cell_id: id,
                                        },
                                        Severity::Error,
                                    );
                                    issues.add(issue);
                                }
                            }
                            None => {
                                let issue = ValidatorIssue::new_and_log(
                                    Cause::UnconnectedPort {
                                        child_cell_id: instance.child.unwrap_cell(),
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
                    }

                    // Check for extra ports
                    for conn in instance.connections.keys() {
                        if !child_ports.contains(conn) {
                            let issue = ValidatorIssue::new_and_log(
                                Cause::ExtraPort {
                                    child_cell_id: instance.child.unwrap_cell(),
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
                }
                ChildId::Primitive(p) => {
                    if self.try_primitive(p).is_none() {
                        let issue = ValidatorIssue::new_and_log(
                            Cause::MissingChild {
                                child_id: p.into(),
                                parent_cell_id: id,
                                parent_cell_name: cell.name.clone(),
                                instance_name: instance.name.clone(),
                            },
                            Severity::Error,
                        );
                        issues.add(issue);
                    }
                }
            }
        }
    }
}
