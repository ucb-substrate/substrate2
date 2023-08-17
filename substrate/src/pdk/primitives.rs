use crate::block::{Block, PdkPrimitive};
use crate::io::{Array, InOut, Signal};
use arcstr::ArcStr;
use indexmap::IndexMap;
use scir::Expr;
use serde::{Deserialize, Serialize};

/// An instance with a pre-defined cell.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawInstance {
    /// The name of the underlying cell.
    cell: ArcStr,
    /// The name of the ports of the underlying cell.
    ports: Vec<ArcStr>,
    /// The parameters to pass to the instance.
    params: IndexMap<ArcStr, Expr>,
}
impl RawInstance {
    /// Create a new raw instance with the given parameters.
    #[inline]
    pub fn from_params(
        cell: ArcStr,
        ports: Vec<ArcStr>,
        params: impl Into<IndexMap<ArcStr, Expr>>,
    ) -> Self {
        Self {
            cell,
            ports,
            params: params.into(),
        }
    }
    /// Create a new raw instance with no parameters.
    #[inline]
    pub fn new(cell: ArcStr, ports: Vec<ArcStr>) -> Self {
        Self {
            cell,
            ports,
            params: IndexMap::new(),
        }
    }
}
impl Block for RawInstance {
    type Kind = PdkPrimitive;
    type Io = InOut<Array<Signal>>;

    fn id() -> ArcStr {
        arcstr::literal!("raw_instance")
    }

    fn name(&self) -> ArcStr {
        arcstr::format!("raw_instance_{}", self.cell)
    }

    fn io(&self) -> Self::Io {
        InOut(Array::new(self.ports.len(), Default::default()))
    }
}
