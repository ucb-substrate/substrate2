//! Spectre analyses.

use serde::{Deserialize, Serialize};

pub mod ac;
pub mod dc;
pub mod montecarlo;
pub mod tran;

/// Sweep kinds.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Sweep {
    /// Linear sweep with the given number of points.
    Linear(usize),
    /// Logarithmic sweep with the given number of points.
    Logarithmic(usize),
    /// Logarithmic sweep with the given number of points **per decade**.
    Decade(usize),
}
