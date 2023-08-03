//! SCIR driver validation.
//!
//! Looks for issues such as multiply-driven nets and floating nets.

use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

use diagnostics::{Diagnostic, IssueSet, Severity};

use super::*;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Net {
    cell_name: ArcStr,
    signal_name: ArcStr,
    idx: Option<usize>,
}

impl Display for Net {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.cell_name, self.signal_name)?;
        if let Some(idx) = self.idx {
            write!(f, "[{idx}]")?;
        }
        Ok(())
    }
}

/// An issue identified during validation of an SCIR library.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct DriverIssue {
    cause: Cause,
    severity: Severity,
    net: Net,
}

/// The state of a net.
#[derive(Debug, Clone, Default)]
struct NetState {
    /// The number of drivers on this net.
    ///
    /// A module input port counts as a driver,
    /// since an input port is presumably driven
    /// by some other module.
    drivers: usize,
    /// The number of "readers" on this net.
    ///
    /// A module output port counts as a tap,
    /// since an output port is presumably read
    /// by some other module.
    taps: usize,
    /// The number of inouts connected to this net.
    ///
    /// A module inout port counts as a tap.
    inouts: usize,
}

impl NetState {
    /// Creates a new [`NetState`].
    #[inline]
    fn new() -> Self {
        Self::default()
    }

    /// Returns the number of connections to the net.
    fn degree(&self) -> usize {
        self.drivers + self.taps + self.inouts
    }

    /// Returns the effective number of drivers of the net.
    ///
    /// Counts inouts as drivers.
    fn eff_drivers(&self) -> usize {
        self.inouts + self.drivers
    }

    /// Validates the number of drivers, taps, and inouts on the net.
    fn validate(&self, net: Net, output: &mut IssueSet<DriverIssue>) {
        if self.drivers > 1 {
            output.add(DriverIssue::new_and_log(
                Cause::MultipleDrivers,
                net.clone(),
                Severity::Info,
            ));
        }

        if self.taps > 0 && self.inouts + self.drivers == 0 {
            output.add(DriverIssue::new_and_log(
                Cause::NoDrivers,
                net.clone(),
                Severity::Warning,
            ));
        }

        if self.degree() == 0 {
            output.add(DriverIssue::new_and_log(
                Cause::Floating,
                net.clone(),
                Severity::Warning,
            ));
        }

        if self.taps == 0 && self.eff_drivers() == 1 {
            output.add(DriverIssue::new_and_log(
                Cause::NotConnected,
                net.clone(),
                Severity::Info,
            ));
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Cause {
    /// A net that is driven but not tapped.
    ///
    /// Example: the output of an inverter is left unconnected.
    NotConnected,
    /// A net that is declared but not connected to anything.
    Floating,
    /// A net has multiple drivers.
    ///
    /// This may not be an issue in all contexts.
    ///
    /// Example: two inverters drive the same output net.
    MultipleDrivers,
    /// A net that is used, but has no drivers.
    ///
    /// Example: an inverter whose input port is not connected.
    NoDrivers,
}

impl Diagnostic for DriverIssue {
    fn severity(&self) -> Severity {
        self.severity
    }
}

impl DriverIssue {
    /// Creates a new validator issue from the given cause and severity.
    pub(crate) fn new(cause: Cause, net: Net, severity: Severity) -> Self {
        Self {
            cause,
            net,
            severity,
        }
    }

    /// Gets the underlying cause of this issue.
    #[inline]
    pub fn cause(&self) -> &Cause {
        &self.cause
    }

    /// Creates a new validator issue and logs it immediately.
    ///
    /// The log level will be selected according to the given severity.
    pub(crate) fn new_and_log(cause: Cause, net: Net, severity: Severity) -> Self {
        let result = Self::new(cause, net, severity);
        match severity {
            Severity::Info => tracing::event!(Level::INFO, issue = ?result.cause, "{}", result),
            Severity::Warning => tracing::event!(Level::WARN, issue = ?result.cause, "{}", result),
            Severity::Error => tracing::event!(Level::ERROR, issue = ?result.cause, "{}", result),
        }
        result
    }
}

impl Display for DriverIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.cause, self.net)
    }
}

impl Display for Cause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Floating => write!(f, "floating net"),
            Self::MultipleDrivers => write!(f, "multiple drivers on the same net"),
            Self::NoDrivers => write!(f, "net is used (i.e. read from), but has no drivers"),
            Self::NotConnected => write!(f, "net is driven but never used elsewhere"),
        }
    }
}

impl Library {
    /// Check whether or not this library is valid.
    pub fn validate_drivers(&self) -> IssueSet<DriverIssue> {
        let _guard = span!(Level::INFO, "performing driver analysis on SCIR Library").entered();
        let mut issues = IssueSet::new();
        self.validate_drivers_inner(&mut issues);
        issues
    }

    fn validate_drivers_inner(&self, issues: &mut IssueSet<DriverIssue>) {
        let _guard = span!(
            Level::INFO,
            "validation pass 1 (checking signal and port identifier validity)"
        )
        .entered();

        for (id, cell) in self.cells.iter() {
            self.validate_cell_drivers(*id, issues);
        }
    }

    fn validate_cell_drivers(&self, id: CellId, issues: &mut IssueSet<DriverIssue>) {
        let cell = self.cells.get(&id).unwrap();
        let _guard =
            span!(Level::INFO, "validating SCIR cell drivers", cell.id = %id, cell.name = %cell.name)
                .entered();

        // Cannot validate blackbox cells.
        if cell.contents().is_opaque() {
            return;
        }
        let contents = cell.contents().as_ref().unwrap_clear();

        let mut net_states: HashMap<SignalId, Vec<NetState>> =
            HashMap::from_iter(cell.signals().map(|(id, info)| {
                let len = info.width.unwrap_or(1);
                (id, vec![NetState::new(); len])
            }));

        for port in cell.ports() {
            for state in net_states[&port.signal()].iter_mut() {
                match port.direction {
                    Direction::Input => state.drivers += 1,
                    Direction::Output => state.taps += 1,
                    Direction::InOut => state.inouts += 1,
                }
            }
        }

        for (_, instance) in contents.instances.iter() {
            analyze_instance(self, &mut net_states, instance);
        }

        for device in contents.primitives.iter() {
            for slice in device.nodes() {}
        }
    }
}

fn analyze_instance(
    lib: &Library,
    net_states: &mut HashMap<SignalId, Vec<NetState>>,
    inst: &Instance,
) {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::*;
    use test_log::test;

    #[test]
    fn duplicate_cell_names() {
        let c1 = Cell::new_whitebox("duplicate_cell_name");
        let mut c2 = Cell::new_blackbox("duplicate_cell_name");
        c2.add_blackbox_elem("* contents of cell");
        let mut lib = Library::new("duplicate_cell_names");
        lib.add_cell(c1);
        lib.add_cell(c2);
        let issues = lib.validate();
        assert!(issues.has_error() || issues.has_warning());
    }
}
