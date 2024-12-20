use arcstr::ArcStr;
use gds::{
    GdsBoundary, GdsElement, GdsLibrary, GdsPoint, GdsStrans, GdsStruct, GdsStructRef, GdsTextElem,
    GdsUnits,
};
use geometry::{
    corner::Corner,
    point::Point,
    prelude::{Orientation, Polygon},
    rect::Rect,
};
use layir::{Cell, Element, Instance, Library, LibraryBuilder, Shape, Text};
use serde::{Deserialize, Serialize};

pub mod export;
pub mod import;

#[cfg(test)]
mod tests;

/// A GDS layer specification.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct GdsLayer(pub u16, pub u16);
