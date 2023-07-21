use std::marker::PhantomData;

use geometry::{
    align::AlignRectMut,
    prelude::{AlignMode, Bbox},
    rect::Rect,
    transform::TranslateMut,
};
use serde::{Deserialize, Serialize};

use crate::pdk::Pdk;

use super::{Draw, DrawContainer, DrawRef};

pub trait Tileable<PDK: Pdk>: Draw<PDK> + AlignRectMut {}
impl<PDK: Pdk, T: Draw<PDK> + AlignRectMut> Tileable<PDK> for T {}
pub trait RefTileable<PDK: Pdk>: DrawRef<PDK> + AlignRectMut {}
impl<PDK: Pdk, T: DrawRef<PDK> + AlignRectMut> RefTileable<PDK> for T {}

pub struct Tile<'a, PDK: Pdk> {
    inner: TileInner<'a, PDK>,
}

enum TileInner<'a, PDK: Pdk> {
    Occupied(OccupiedTile<'a, PDK>),
    Empty,
}

struct OccupiedTile<'a, PDK: Pdk> {
    kind: TileKind<'a, PDK>,
    rect: Rect,
}

enum TileKind<'a, PDK: Pdk> {
    Owned(Box<dyn Tileable<PDK> + 'a>),
    Ref(&'a mut dyn RefTileable<PDK>),
}

impl<'a, PDK: Pdk> TranslateMut for TileKind<'a, PDK> {
    fn translate_mut(&mut self, p: geometry::prelude::Point) {
        match self {
            Self::Owned(tile) => tile.translate_mut(p),
            Self::Ref(ref_tile) => ref_tile.translate_mut(p),
        }
    }
}

impl<'a, PDK: Pdk> Draw<PDK> for TileKind<'a, PDK> {
    fn draw(self, cell: &mut DrawContainer<PDK>) -> crate::error::Result<()> {
        match self {
            Self::Owned(tile) => tile.draw(cell)?,
            Self::Ref(ref_tile) => ref_tile.draw_ref(cell)?,
        }
        Ok(())
    }
}

impl<'a, PDK: Pdk> TranslateMut for OccupiedTile<'a, PDK> {
    fn translate_mut(&mut self, p: geometry::prelude::Point) {
        self.kind.translate_mut(p);
        self.rect.translate_mut(p);
    }
}

impl<'a, PDK: Pdk> Draw<PDK> for OccupiedTile<'a, PDK> {
    fn draw(self, cell: &mut DrawContainer<PDK>) -> crate::error::Result<()> {
        self.kind.draw(cell)
    }
}

impl<'a, PDK: Pdk> Tile<'a, PDK> {
    pub fn new<T: 'a + Tileable<PDK>>(inner: T, rect: Rect) -> Self {
        Self {
            inner: TileInner::Occupied(OccupiedTile {
                kind: TileKind::Owned(Box::new(inner)),
                rect,
            }),
        }
    }

    pub fn new_ref<T: RefTileable<PDK>>(inner: &'a mut T, rect: Rect) -> Self {
        Self {
            inner: TileInner::Occupied(OccupiedTile {
                kind: TileKind::Ref(inner),
                rect,
            }),
        }
    }

    pub fn from_bbox<T: 'a + Bbox + Tileable<PDK>>(inner: T) -> Self {
        let rect = inner.bbox().unwrap();
        Self {
            inner: TileInner::Occupied(OccupiedTile {
                kind: TileKind::Owned(Box::new(inner)),
                rect,
            }),
        }
    }

    pub fn from_bbox_ref<T: Bbox + RefTileable<PDK> + 'a>(inner: &'a mut T) -> Self {
        let rect = inner.bbox().unwrap();
        Self {
            inner: TileInner::Occupied(OccupiedTile {
                kind: TileKind::Ref(inner),
                rect,
            }),
        }
    }

    pub fn empty() -> Self {
        Self {
            inner: TileInner::Empty,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileAlignMode {
    PosFlush,
    PosAdjacent,
    NegFlush,
    NegAdjacent,
    Center,
}

pub struct ArrayTiler<'a, PDK: Pdk> {
    config: ArrayTilerConfig,
    tiles: Vec<Tile<'a, PDK>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrayTilerConfig {
    horiz_mode: TileAlignMode,
    horiz_offset: i64,
    vert_mode: TileAlignMode,
    vert_offset: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrayTilerBuilder<PDK> {
    phantom: PhantomData<PDK>,
    horiz_mode: Option<TileAlignMode>,
    horiz_offset: Option<i64>,
    vert_mode: Option<TileAlignMode>,
    vert_offset: Option<i64>,
}

impl<PDK> Default for ArrayTilerBuilder<PDK> {
    fn default() -> Self {
        Self {
            phantom: PhantomData,
            horiz_mode: None,
            horiz_offset: None,
            vert_mode: None,
            vert_offset: None,
        }
    }
}

impl<PDK: Pdk> ArrayTilerBuilder<PDK> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn horiz_mode(&mut self, mode: TileAlignMode) -> &mut Self {
        self.horiz_mode = Some(mode);
        self
    }

    pub fn horiz_offset(&mut self, offset: i64) -> &mut Self {
        self.horiz_offset = Some(offset);
        self
    }

    pub fn vert_mode(&mut self, mode: TileAlignMode) -> &mut Self {
        self.vert_mode = Some(mode);
        self
    }

    pub fn vert_offset(&mut self, offset: i64) -> &mut Self {
        self.vert_offset = Some(offset);
        self
    }

    pub fn build<'a>(&mut self) -> ArrayTiler<'a, PDK> {
        ArrayTiler {
            config: ArrayTilerConfig {
                horiz_mode: self.horiz_mode.unwrap(),
                horiz_offset: self.horiz_offset.unwrap_or_default(),
                vert_mode: self.vert_mode.unwrap(),
                vert_offset: self.vert_offset.unwrap_or_default(),
            },
            tiles: Vec::new(),
        }
    }
}

impl<'a, PDK: Pdk> ArrayTiler<'a, PDK> {
    pub fn builder() -> ArrayTilerBuilder<PDK> {
        ArrayTilerBuilder::new()
    }

    pub fn push(&mut self, tile: Tile<'a, PDK>) {
        self.tiles.push(tile);
    }

    pub fn apply(&mut self) {
        let mut prev_rect = None;
        for tile in self.tiles.iter_mut() {
            if let TileInner::Occupied(tile) = &mut tile.inner {
                if let Some(prev_rect) = prev_rect {
                    let srect = tile.rect;
                    ArrayTiler::align_with_prev(tile, &self.config, srect, prev_rect);
                }
                prev_rect = Some(tile.rect);
            }
        }
    }

    fn align_with_prev(
        tile: &mut OccupiedTile<'a, PDK>,
        config: &ArrayTilerConfig,
        srect: Rect,
        orect: Rect,
    ) {
        tile.align_mut(
            match config.horiz_mode {
                TileAlignMode::PosFlush => AlignMode::Right,
                TileAlignMode::PosAdjacent => AlignMode::ToTheRight,
                TileAlignMode::Center => AlignMode::CenterHorizontal,
                TileAlignMode::NegFlush => AlignMode::Left,
                TileAlignMode::NegAdjacent => AlignMode::ToTheLeft,
            },
            srect,
            orect,
            config.horiz_offset,
        );
        tile.align_mut(
            match config.vert_mode {
                TileAlignMode::PosFlush => AlignMode::Top,
                TileAlignMode::PosAdjacent => AlignMode::Above,
                TileAlignMode::Center => AlignMode::CenterVertical,
                TileAlignMode::NegFlush => AlignMode::Bottom,
                TileAlignMode::NegAdjacent => AlignMode::Beneath,
            },
            srect,
            orect,
            config.vert_offset,
        );
    }
}

impl<'a, PDK: Pdk> Draw<PDK> for ArrayTiler<'a, PDK> {
    fn draw(self, cell: &mut DrawContainer<PDK>) -> crate::error::Result<()> {
        let mut prev_rect = None;
        for tile in self.tiles {
            if let TileInner::Occupied(mut tile) = tile.inner {
                if let Some(prev_rect) = prev_rect {
                    let srect = tile.rect;
                    ArrayTiler::align_with_prev(&mut tile, &self.config, srect, prev_rect);
                }
                prev_rect = Some(tile.rect);
                cell.draw(tile)?;
            }
        }
        Ok(())
    }
}
