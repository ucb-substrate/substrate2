//! SCIR driver validation.
//!
//! Looks for issues such as multiply-driven nets and floating nets.

use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use diagnostics::{Diagnostic, IssueSet, Severity};

use super::*;

/// A single node in a SCIR circuit.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Net {
    /// The name of the cell containing this net.
    cell_name: ArcStr,
    /// The name of the signal.
    signal_name: ArcStr,
    /// The signal bit index, if the signal is a bus.
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

/// The cause of a driver analysis error or warning.
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

impl<S: Schema + ?Sized> LibraryBuilder<S> {
    /// Perform driver analysis on this library.
    pub fn validate_drivers(&self) -> IssueSet<DriverIssue> {
        let _guard = span!(Level::INFO, "performing driver analysis on SCIR Library").entered();
        let mut issues = IssueSet::new();
        self.validate_drivers_inner(&mut issues);
        issues
    }

    fn validate_drivers_inner(&self, issues: &mut IssueSet<DriverIssue>) {
        for &id in self.cells.keys() {
            self.validate_cell_drivers(id, issues);
        }
    }

    fn validate_cell_drivers(&self, id: CellId, issues: &mut IssueSet<DriverIssue>) {
        let cell = self.cells.get(&id).unwrap();
        let _guard =
            span!(Level::INFO, "validating SCIR cell drivers", cell.id = %id, cell.name = %cell.name)
                .entered();

        let mut net_states: HashMap<SignalId, Vec<NetState>> =
            HashMap::from_iter(cell.signals().map(|(id, info)| {
                let len = info.width.unwrap_or(1);
                (id, vec![NetState::new(); len])
            }));

        for port in cell.ports() {
            for state in net_states.get_mut(&port.signal()).unwrap() {
                match port.direction {
                    Direction::Input => state.drivers += 1,
                    Direction::Output => state.taps += 1,
                    Direction::InOut => state.inouts += 1,
                }
            }
        }

        for (_, instance) in cell.instances.iter() {
            analyze_instance(self, &mut net_states, instance);
        }

        for (sig, list) in net_states.iter() {
            for (i, state) in list.iter().enumerate() {
                let info = cell.signal(*sig);
                state.validate(
                    Net {
                        cell_name: cell.name().clone(),
                        signal_name: info.name.clone(),
                        idx: info.width.map(|_| i),
                    },
                    issues,
                );
            }
        }
    }
}

fn analyze_instance<S: Schema + ?Sized>(
    lib: &LibraryBuilder<S>,
    net_states: &mut HashMap<SignalId, Vec<NetState>>,
    inst: &Instance,
) {
    if inst.child().is_primitive() {
        return;
    }
    let cell = lib.cell(inst.child().unwrap_cell());
    for (port, conn) in inst.connections() {
        let dir = cell.port(port).direction;
        for part in conn.parts() {
            let states = net_states.get_mut(&part.signal()).unwrap();
            if let Some(range) = part.range() {
                for idx in range {
                    update_net_state(&mut states[idx], dir);
                }
            } else {
                update_net_state(&mut states[0], dir);
            }
        }
    }
}

fn update_net_state(state: &mut NetState, dir: Direction) {
    match dir {
        Direction::Output => state.drivers += 1,
        Direction::Input => state.taps += 1,
        Direction::InOut => state.inouts += 1,
    }
}
