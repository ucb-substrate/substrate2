//! Utilities for using Atoll with the SKY130 technology.
//!
//! # Metal Layers
//!
//! | Layer | Direction | Line (nm) | Space (nm) | Pitch (nm) |
//! |---------|---------|---------|---------|---------|
//! | M0 (LICON) | Any | 170 | 170 | 340 |
//! | M1 | H | 260 | 140 | 400 |
//! | M2 | V | 260 | 140 | 400 |
//! | M3 | H | 320 | 320 | 640 |
//!
//! Tile width: LCM(340, 400) = 6800
//! Tile height: LCM(400, 620) = 3200

use std::io::BufRead;
use crate::mos::MosKind;
use crate::{ConvError, Sky130Pdk};
use scir::schema::Schema;
use scir::{Instance, Library};
use serde::{Deserialize, Serialize};
use substrate::block::Block;
use substrate::geometry::rect::Rect;
use substrate::io::LayoutType;
use substrate::layout::element::Shape;
use substrate::layout::{CellBuilder, ExportsLayoutData, Layout};
use substrate::schematic::schema::Primitive;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Block)]
#[substrate(io = "()", kind = "Cell")]
pub struct NmosTile {}

impl ExportsLayoutData for NmosTile {
    type LayoutData = ();
}

impl Layout<Sky130Pdk> for NmosTile {
    fn layout(
        &self,
        io: &mut <<Self as Block>::Io as LayoutType>::Builder,
        cell: &mut CellBuilder<Sky130Pdk, Self>,
    ) -> substrate::error::Result<Self::LayoutData> {
        let layers = &cell.ctx.layers;
        cell.draw(Shape::new(layers.met1, Rect::from_sides(0, 0, 6800, 3200)))?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Sky130Tile {
    Nmos(NmosTile),
}

impl Schema for Sky130Tile {
    type Primitive = Sky130Tile;
}

impl atoll::Tile for Sky130Tile {
    fn rowspan(&self) -> usize {
        1
    }

    fn colspan(&self) -> usize {
        1
    }
}

impl scir::schema::FromSchema<Sky130Pdk> for Sky130Tile {
    type Error = ConvError;

    fn convert_primitive(
        primitive: <Sky130Pdk as Schema>::Primitive,
    ) -> Result<<Self as Schema>::Primitive, Self::Error> {
        match primitive {
            crate::Primitive::RawInstance { .. } => Err(ConvError::UnsupportedPrimitive),
            crate::Primitive::Mos { kind, params } => match kind {
                MosKind::Nfet01v8 => Ok(Sky130Tile::Nmos(NmosTile {})),
                _ => Err(ConvError::UnsupportedPrimitive),
            },
        }
    }

    fn convert_instance(
        instance: &mut Instance,
        primitive: &<Sky130Pdk as Schema>::Primitive,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub struct Sky130Atoll;

impl atoll::Syn<Sky130Pdk, Sky130Tile> for Sky130Atoll {
    fn syn(&self, lib: Library<Sky130Pdk>) -> Library<Sky130Tile> {
        lib.convert_schema().unwrap().build().unwrap()
    }
}
