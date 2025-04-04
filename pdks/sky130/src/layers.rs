//! The set of PDK layers.
#![allow(missing_docs)]

use std::collections::HashMap;

use gdsconv::{conv::FromGds, GdsLayer};
use lazy_static::lazy_static;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Sky130Layer {
    PrBoundary,
    Pwell,
    Nwell,
    Dnwell,
    Vhvi,
    Diff,
    Tap,
    Psdm,
    Nsdm,
    Poly,
    PolyRes,
    PolyCut,
    Ldntm,
    Lvtn,
    Hvtp,
    Hvtr,
    Tunm,
    Licon1,
    /// Nitride poly cut.
    Npc,
    Li1,
    Mcon,
    Met1,
    Via,
    Met2,
    Via2,
    Met3,
    Via3,
    Met4,
    Via4,
    Met5,
    Pad,
    Rpm,
    Urpm,
    Hvi,
    Ncm,
    CfomDrawing,
    CfomMask,
    CfomMaskAdd,
    CfomMaskDrop,
    Cli1mDrawing,
    Cli1mMask,
    Cli1mMaskAdd,
    Cli1mMaskDrop,
    AreaIdLowTapDensity,
    AreaIdSeal,
    AreaIdCore,
    AreaIdFrame,
    AreaIdEsd,
    AreaIdStandardc,
    AreaIdAnalog,
    Outline,
    Text,
}

lazy_static! {
    static ref SKY130_TO_GDS_DRAWING_LAYER: HashMap<Sky130Layer, GdsLayer> = HashMap::from_iter([
        (Sky130Layer::PrBoundary, GdsLayer(235, 4)),
        (Sky130Layer::Pwell, GdsLayer(64, 44)),
        (Sky130Layer::Nwell, GdsLayer(64, 20)),
        (Sky130Layer::Dnwell, GdsLayer(64, 18)),
        (Sky130Layer::Vhvi, GdsLayer(74, 21)),
        (Sky130Layer::Diff, GdsLayer(65, 20)),
        (Sky130Layer::Tap, GdsLayer(65, 44)),
        (Sky130Layer::Psdm, GdsLayer(94, 20)),
        (Sky130Layer::Nsdm, GdsLayer(93, 44)),
        (Sky130Layer::Poly, GdsLayer(66, 20)),
        (Sky130Layer::PolyRes, GdsLayer(66, 13)),
        (Sky130Layer::PolyCut, GdsLayer(66, 14)),
        (Sky130Layer::Ldntm, GdsLayer(11, 44)),
        (Sky130Layer::Lvtn, GdsLayer(125, 44)),
        (Sky130Layer::Hvtp, GdsLayer(78, 44)),
        (Sky130Layer::Hvtr, GdsLayer(18, 20)),
        (Sky130Layer::Tunm, GdsLayer(80, 20)),
        (Sky130Layer::Licon1, GdsLayer(66, 44)),
        (Sky130Layer::Npc, GdsLayer(95, 20)),
        (Sky130Layer::Li1, GdsLayer(67, 20)),
        (Sky130Layer::Mcon, GdsLayer(67, 44)),
        (Sky130Layer::Met1, GdsLayer(68, 20)),
        (Sky130Layer::Via, GdsLayer(68, 44)),
        (Sky130Layer::Met2, GdsLayer(69, 20)),
        (Sky130Layer::Via2, GdsLayer(69, 44)),
        (Sky130Layer::Met3, GdsLayer(70, 20)),
        (Sky130Layer::Via3, GdsLayer(70, 44)),
        (Sky130Layer::Met4, GdsLayer(71, 20)),
        (Sky130Layer::Via4, GdsLayer(71, 44)),
        (Sky130Layer::Met5, GdsLayer(72, 20)),
        (Sky130Layer::Pad, GdsLayer(76, 20)),
        (Sky130Layer::Rpm, GdsLayer(86, 20)),
        (Sky130Layer::Urpm, GdsLayer(79, 20)),
        (Sky130Layer::Hvi, GdsLayer(75, 20)),
        (Sky130Layer::Ncm, GdsLayer(92, 44)),
        (Sky130Layer::CfomDrawing, GdsLayer(22, 20)),
        (Sky130Layer::CfomMask, GdsLayer(23, 0)),
        (Sky130Layer::CfomMaskAdd, GdsLayer(22, 21)),
        (Sky130Layer::CfomMaskDrop, GdsLayer(22, 22)),
        (Sky130Layer::Cli1mDrawing, GdsLayer(115, 44)),
        (Sky130Layer::Cli1mMask, GdsLayer(56, 0)),
        (Sky130Layer::Cli1mMaskAdd, GdsLayer(115, 43)),
        (Sky130Layer::Cli1mMaskDrop, GdsLayer(115, 42)),
        (Sky130Layer::AreaIdLowTapDensity, GdsLayer(81, 14)),
        (Sky130Layer::AreaIdSeal, GdsLayer(81, 1)),
        (Sky130Layer::AreaIdCore, GdsLayer(81, 2)),
        (Sky130Layer::AreaIdFrame, GdsLayer(81, 3)),
        (Sky130Layer::AreaIdEsd, GdsLayer(81, 19)),
        (Sky130Layer::AreaIdStandardc, GdsLayer(81, 4)),
        (Sky130Layer::AreaIdAnalog, GdsLayer(81, 79)),
        (Sky130Layer::Outline, GdsLayer(236, 0)),
        (Sky130Layer::Text, GdsLayer(83, 44)),
    ]);
    static ref SKY130_TO_GDS_PIN_LAYER: HashMap<Sky130Layer, GdsLayer> = HashMap::from_iter([
        (Sky130Layer::Pwell, GdsLayer(122, 16)),
        (Sky130Layer::Nwell, GdsLayer(64, 16)),
        (Sky130Layer::Poly, GdsLayer(66, 16)),
        (Sky130Layer::Licon1, GdsLayer(66, 58)),
        (Sky130Layer::Li1, GdsLayer(67, 16)),
        (Sky130Layer::Mcon, GdsLayer(67, 48)),
        (Sky130Layer::Met1, GdsLayer(68, 16)),
        (Sky130Layer::Via, GdsLayer(68, 58)),
        (Sky130Layer::Met2, GdsLayer(69, 16)),
        (Sky130Layer::Via2, GdsLayer(69, 58)),
        (Sky130Layer::Met3, GdsLayer(70, 16)),
        (Sky130Layer::Via3, GdsLayer(70, 48)),
        (Sky130Layer::Met4, GdsLayer(71, 16)),
        (Sky130Layer::Via4, GdsLayer(71, 48)),
        (Sky130Layer::Met5, GdsLayer(72, 16)),
        (Sky130Layer::Pad, GdsLayer(76, 16)),
    ]);
    static ref SKY130_TO_GDS_LABEL_LAYER: HashMap<Sky130Layer, GdsLayer> = HashMap::from_iter([
        (Sky130Layer::Pwell, GdsLayer(64, 59)),
        (Sky130Layer::Nwell, GdsLayer(64, 5)),
        (Sky130Layer::Poly, GdsLayer(66, 5)),
        (Sky130Layer::Licon1, GdsLayer(66, 41)),
        (Sky130Layer::Li1, GdsLayer(67, 5)),
        (Sky130Layer::Mcon, GdsLayer(67, 41)),
        (Sky130Layer::Met1, GdsLayer(68, 5)),
        (Sky130Layer::Via, GdsLayer(68, 41)),
        (Sky130Layer::Met2, GdsLayer(69, 5)),
        (Sky130Layer::Via2, GdsLayer(69, 41)),
        (Sky130Layer::Met3, GdsLayer(70, 5)),
        (Sky130Layer::Via3, GdsLayer(70, 41)),
        (Sky130Layer::Met4, GdsLayer(71, 5)),
        (Sky130Layer::Via4, GdsLayer(71, 41)),
        (Sky130Layer::Met5, GdsLayer(72, 5)),
        (Sky130Layer::Pad, GdsLayer(76, 5)),
    ]);
    static ref GDS_PIN_LAYER_TO_SKY130: HashMap<GdsLayer, Sky130Layer> =
        HashMap::from_iter(SKY130_TO_GDS_PIN_LAYER.iter().map(|(k, v)| (*v, *k)));
    static ref GDS_LABEL_LAYER_TO_SKY130: HashMap<GdsLayer, Sky130Layer> =
        HashMap::from_iter(SKY130_TO_GDS_LABEL_LAYER.iter().map(|(k, v)| (*v, *k)));
    static ref GDS_LAYER_TO_SKY130: HashMap<GdsLayer, Sky130Layer> = HashMap::from_iter(
        SKY130_TO_GDS_DRAWING_LAYER
            .iter()
            .map(|(k, v)| (*v, *k))
            .chain(SKY130_TO_GDS_PIN_LAYER.iter().map(|(k, v)| (*v, *k)))
            .chain(SKY130_TO_GDS_LABEL_LAYER.iter().map(|(k, v)| (*v, *k)))
    );
}

impl Sky130Layer {
    pub fn gds_layer(&self) -> GdsLayer {
        SKY130_TO_GDS_DRAWING_LAYER[self]
    }

    pub fn gds_pin_layer(&self) -> Option<GdsLayer> {
        SKY130_TO_GDS_PIN_LAYER.get(self).copied()
    }

    pub fn gds_label_layer(&self) -> Option<GdsLayer> {
        SKY130_TO_GDS_LABEL_LAYER.get(self).copied()
    }
}

impl FromGds for Sky130Layer {
    fn from_gds(layer: GdsLayer) -> Option<Self> {
        GDS_LAYER_TO_SKY130.get(&layer).copied()
    }

    fn from_gds_pin(layer: GdsLayer) -> Option<Self> {
        GDS_PIN_LAYER_TO_SKY130.get(&layer).copied()
    }

    fn from_gds_label(layer: GdsLayer) -> Option<Self> {
        GDS_LABEL_LAYER_TO_SKY130.get(&layer).copied()
    }
}
