//! Tile definitions.

use substrate::types::{InOut, Io, Signal};

/// MOS device kind.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum MosKind {
    /// Nominal Vt.
    Nom,
    /// Low Vt.
    Lvt,
    /// Ultra low Vt.
    Ulvt,
}

/// The IO of a tap.
#[derive(Default, Debug, Clone, Copy, Io)]
pub struct TapIo {
    /// The tap contact.
    pub x: InOut<Signal>,
}

/// The kind of tile.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum TileKind {
    /// An n-type tile.
    N,
    /// A p-type tile.
    P,
}

/// MOS tile parameters.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct MosTileParams {
    /// The flavor of MOS device.
    pub mos_kind: MosKind,
    /// Whether MOS is n-channel or p-channel.
    pub tile_kind: TileKind,
    /// The MOS device width.
    pub w: i64,
}

impl MosTileParams {
    /// Creates a new [`MosTileParams`].
    pub fn new(mos_kind: MosKind, tile_kind: TileKind, w: i64) -> Self {
        Self {
            mos_kind,
            tile_kind,
            w,
        }
    }
}

/// Tap tile parameters.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct TapTileParams {
    /// The kind of tap.
    pub kind: TileKind,
    /// Number of MOS devices this tap must span.
    pub mos_span: i64,
}

impl TapTileParams {
    /// Creates a new [`TapTileParams`].
    pub fn new(kind: TileKind, mos_span: i64) -> Self {
        Self { kind, mos_span }
    }
}

/// The IO of a resistor.
#[derive(Default, Debug, Clone, Copy, Io)]
pub struct ResistorIo {
    /// The positive terminal.
    pub p: InOut<Signal>,
    /// The negative terminal.
    pub n: InOut<Signal>,
    /// The body terminal.
    pub b: InOut<Signal>,
}

/// Resistor tile parameters.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ResistorTileParams {
    /// Resistor length.
    pub l: i64,
}

impl ResistorTileParams {
    /// Creates a new [`ResistorTileParams`].
    pub fn new(l: i64) -> Self {
        Self { l }
    }
}

/// Resistor connection configurations.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ResistorConn {
    /// Series.
    Series,
    /// Parallel.
    Parallel,
}
