//! Tiling structures and helpers.

use std::marker::PhantomData;

use downcast_rs::{impl_downcast, Downcast};
use geometry::{
    align::AlignRectMut,
    prelude::{AlignMode, Bbox},
    rect::Rect,
    side::Sides,
    transform::TranslateMut,
};
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};

use crate::pdk::Pdk;

use super::{Draw, DrawReceiver};

/// A tileable layout object.
pub trait Tileable<PDK: Pdk>: Draw<PDK> + AlignRectMut + Downcast {}
impl<PDK: Pdk, T: Draw<PDK> + AlignRectMut + Downcast> Tileable<PDK> for T {}
impl_downcast!(Tileable<PDK> where PDK: Pdk);

new_key_type! {
    struct RawTileKey;
}

/// A key for indexing a [`Tile`] within an [`ArrayTiler`].
pub struct ArrayTileKey<T> {
    key: RawTileKey,
    phantom: PhantomData<T>,
}

impl<T> Clone for ArrayTileKey<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ArrayTileKey<T> {}

/// A tile container for a [`Tileable`] object.
#[derive(Debug, Clone, Copy)]
pub struct Tile<T> {
    /// The [`Tileable`] object to be tiled.
    pub inner: T,
    /// A rectangle used for alignment with other [`Tile`]s.
    pub rect: Rect,
}

struct RawTile<PDK: Pdk> {
    inner: Box<dyn Tileable<PDK>>,
    rect: Rect,
}

impl<T> Tile<T> {
    /// Creates a new [`Tile`].
    pub fn new(inner: T, rect: Rect) -> Self {
        Self { inner, rect }
    }

    /// Returns a new [`Tile`] with the given padding on each side.
    pub fn with_padding(mut self, padding: Sides<i64>) -> Self {
        self.rect = self.rect.expand_sides(padding);
        self
    }
}

impl<T: Bbox> Tile<T> {
    /// Creates a new [`Tile`] from the bounding box of the provided layout object.
    pub fn from_bbox(inner: T) -> Self {
        let rect = inner.bbox().unwrap();
        Self { inner, rect }
    }
}

impl<PDK: Pdk, T: Tileable<PDK>> From<Tile<T>> for RawTile<PDK> {
    fn from(value: Tile<T>) -> Self {
        Self {
            inner: Box::new(value.inner),
            rect: value.rect,
        }
    }
}

impl<PDK: Pdk> TranslateMut for RawTile<PDK> {
    fn translate_mut(&mut self, p: geometry::prelude::Point) {
        self.inner.translate_mut(p);
        self.rect.translate_mut(p);
    }
}

impl<PDK: Pdk> Draw<PDK> for RawTile<PDK> {
    fn draw(self, recv: &mut DrawReceiver<PDK>) -> crate::error::Result<()> {
        self.inner.draw(recv)
    }
}

/// An enumeration of alignment modes for adjacent tiles in a tiler along a given axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileAlignMode {
    /// Aligns the positive edge of the next tile to the positive edge of the previous tile.
    PosFlush,
    /// Aligns the negative edge of the next tile to the positive edge of the previous tile.
    PosAdjacent,
    /// Aligns the negative edge of the next tile to the negative edge of the previous tile.
    NegFlush,
    /// Aligns the positive edge of the next tile to the negative edge of the previous tile.
    NegAdjacent,
    /// Aligns the centers of the two tiles along the given axis.
    Center,
}

/// An array tiler.
pub struct ArrayTiler<PDK: Pdk> {
    config: ArrayTilerConfig,
    tiles: SlotMap<RawTileKey, RawTile<PDK>>,
    array: Vec<RawTileKey>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ArrayTilerConfig {
    horiz_mode: TileAlignMode,
    vert_mode: TileAlignMode,
}

impl<PDK: Pdk, T: Tileable<PDK>> std::ops::Index<ArrayTileKey<T>> for ArrayTiler<PDK> {
    type Output = T;

    fn index(&self, index: ArrayTileKey<T>) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<PDK: Pdk> Draw<PDK> for ArrayTiler<PDK> {
    fn draw(mut self, cell: &mut DrawReceiver<PDK>) -> crate::error::Result<()> {
        for key in self.array {
            self.tiles.remove(key).unwrap().draw(cell)?;
        }
        Ok(())
    }
}

impl<PDK: Pdk> ArrayTiler<PDK> {
    /// Creates an [`ArrayTiler`].
    ///
    /// `horiz_mode` and `vert_mode` specify how to align tiles in the horizontal and vertical
    /// directions, respectively.
    pub fn new(horiz_mode: TileAlignMode, vert_mode: TileAlignMode) -> Self {
        Self {
            config: ArrayTilerConfig {
                horiz_mode,
                vert_mode,
            },
            tiles: SlotMap::with_key(),
            array: Vec::new(),
        }
    }

    /// Pushes a new tile to the tiler, returning a key for accessing the tiled object.
    pub fn push<T: Tileable<PDK>>(&mut self, tile: Tile<T>) -> ArrayTileKey<T> {
        let mut raw_tile: RawTile<_> = tile.into();
        if let Some(key) = self.array.last() {
            let srect = raw_tile.rect;
            ArrayTiler::align_with_prev(&mut raw_tile, &self.config, srect, self.tiles[*key].rect);
        }
        let key = self.tiles.insert(raw_tile);
        self.array.push(key);
        ArrayTileKey {
            key,
            phantom: PhantomData,
        }
    }

    /// Pushes a `num` repetitions of a the given tile to the tiler, returning a set of keys to the
    /// new tiled objects.
    pub fn push_num<T: Tileable<PDK> + Clone>(
        &mut self,
        tile: Tile<T>,
        num: usize,
    ) -> Vec<ArrayTileKey<T>> {
        (0..num).map(|_| self.push(tile.clone())).collect()
    }

    /// Pushes each tile in the provided iterator to the tiler, returning an iterator of keys to the new
    /// tiled objects.
    pub fn push_iter<'a, T: Tileable<PDK>>(
        &'a mut self,
        tiles: impl Iterator<Item = Tile<T>> + 'a,
    ) -> impl Iterator<Item = ArrayTileKey<T>> + 'a {
        tiles.into_iter().map(|tile| self.push(tile))
    }

    /// Gets a tiled object using its [`ArrayTileKey`].
    pub fn get<T: Tileable<PDK>>(&self, key: ArrayTileKey<T>) -> Option<&T> {
        self.tiles
            .get(key.key)
            .and_then(|raw| raw.inner.as_ref().downcast_ref())
    }

    fn align_with_prev(
        tile: &mut RawTile<PDK>,
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
            0,
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
            0,
        );
    }
}

/// A key for indexing a [`GridTile`] within an [`GridTiler`].
pub struct GridTileKey<T> {
    key: RawTileKey,
    phantom: PhantomData<T>,
}

impl<T> Clone for GridTileKey<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for GridTileKey<T> {}

/// A tile within a [`GridTiler`].
#[derive(Debug, Clone, Copy)]
pub struct GridTile<T> {
    tile: Option<Tile<T>>,
    colspan: usize,
    rowspan: usize,
}

struct RawGridTile<PDK: Pdk> {
    raw: Option<RawTile<PDK>>,
    colspan: usize,
    rowspan: usize,
}

impl<T> GridTile<T> {
    /// Creates a new [`GridTile`].
    pub fn new(tile: Tile<T>) -> Self {
        Self {
            tile: Some(tile),
            colspan: 1,
            rowspan: 1,
        }
    }

    /// Creates a new empty [`GridTile`].
    pub fn empty() -> Self {
        Self {
            tile: None,
            colspan: 1,
            rowspan: 1,
        }
    }

    /// Returns a new [`GridTile`] with the given column span.
    pub fn with_colspan(mut self, colspan: usize) -> Self {
        self.colspan = colspan;
        self
    }

    /// Returns a new [`GridTile`] with the row span.
    pub fn with_rowspan(mut self, rowspan: usize) -> Self {
        self.rowspan = rowspan;
        self
    }
}

impl<T> From<Tile<T>> for GridTile<T> {
    fn from(value: Tile<T>) -> Self {
        Self::new(value)
    }
}

impl<PDK: Pdk, T: Tileable<PDK>> From<GridTile<T>> for RawGridTile<PDK> {
    fn from(value: GridTile<T>) -> Self {
        Self {
            raw: value.tile.map(|x| x.into()),
            colspan: value.colspan,
            rowspan: value.rowspan,
        }
    }
}

/// A grid tiler.
pub struct GridTiler<PDK: Pdk> {
    #[allow(dead_code)]
    config: GridTilerConfig,
    tiles: SlotMap<RawTileKey, RawGridTile<PDK>>,
    grid: Vec<Vec<RawTileKey>>,
}

impl<PDK: Pdk> Default for GridTiler<PDK> {
    fn default() -> Self {
        Self {
            config: GridTilerConfig {},
            tiles: SlotMap::with_key(),
            grid: vec![vec![]],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct GridTilerConfig {}

/// A constraint on a grid row or column.
#[derive(Debug, Clone, Copy)]
struct GridConstraint {
    /// The constraining row/column index.
    start_index: usize,
    /// The constrained row/column index.
    end_index: usize,
    /// The required minimum distance between the rows/columns.
    distance: i64,
}

#[derive(Debug, Clone, Default)]
struct GridConstraintSolver {
    constraints: Vec<GridConstraint>,
}

impl GridConstraint {
    fn new(start_index: usize, end_index: usize, distance: i64) -> Self {
        assert!(start_index < end_index);
        assert!(distance > 0);
        Self {
            start_index,
            end_index,
            distance,
        }
    }
}

impl GridConstraintSolver {
    fn new() -> Self {
        Self::default()
    }

    fn add(&mut self, constraint: GridConstraint) {
        self.constraints.push(constraint);
    }

    fn solve(mut self) -> Vec<i64> {
        self.constraints
            .sort_by_key(|constraint| constraint.end_index);
        let mut grids = vec![0];

        for constraint in self.constraints {
            if constraint.end_index >= grids.len() {
                grids.resize(constraint.end_index + 1, 0);
            }
            grids[constraint.end_index] = std::cmp::max(
                grids[constraint.end_index],
                grids[constraint.start_index] + constraint.distance,
            );
        }

        grids
    }
}

/// An immutable tiled grid created by a [`GridTiler`].
pub struct TiledGrid<PDK: Pdk> {
    tiles: SlotMap<RawTileKey, RawGridTile<PDK>>,
}

impl<PDK: Pdk> GridTiler<PDK> {
    /// Creates a [`GridTiler`].
    ///
    /// Populated from left-to-right and top-to-bottom.
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes a new tile to the tiler, returning a key for accessing the tiled object.
    pub fn push<T: Tileable<PDK>>(&mut self, tile: impl Into<GridTile<T>>) -> GridTileKey<T> {
        let raw_tile: RawGridTile<_> = tile.into().into();
        let key = self.tiles.insert(raw_tile);
        self.last_row_mut().push(key);
        GridTileKey {
            key,
            phantom: PhantomData,
        }
    }

    /// Pushes a `num` repetitions of a the given tile to the tiler, returning a set of keys to the
    /// new tiled objects.
    pub fn push_num<T: Tileable<PDK> + Clone>(
        &mut self,
        tile: impl Into<GridTile<T>>,
        num: usize,
    ) -> Vec<GridTileKey<T>> {
        let tile = tile.into();
        (0..num).map(|_| self.push(tile.clone())).collect()
    }

    /// Pushes each tile in the provided iterator to the tiler, returning an iterator of keys to the new
    /// tiled objects.
    pub fn push_iter<'a, T: Tileable<PDK>>(
        &'a mut self,
        tiles: impl Iterator<Item = impl Into<GridTile<T>>> + 'a,
    ) -> impl Iterator<Item = GridTileKey<T>> + 'a {
        tiles.into_iter().map(|tile| self.push(tile))
    }

    fn last_row_mut(&mut self) -> &mut Vec<RawTileKey> {
        self.grid.last_mut().unwrap()
    }

    /// Ends a row of the tiler, starting a new one.
    pub fn end_row(&mut self) {
        self.grid.push(Vec::new());
    }

    /// Calculate the row and column indices of each tile, with the necessary shifts applied.
    fn calculate_indices(&self) -> Vec<(usize, usize, RawTileKey)> {
        struct ColBlockage {
            start_col: usize,
            /// Exclusive
            end_col: usize,
            /// Exclusive
            end_row: usize,
        }

        let mut blockages: Vec<ColBlockage> = Vec::new();
        let mut indices = Vec::new();

        for (i, row) in self.grid.iter().enumerate() {
            let mut blockage_idx = 0;
            let mut col_idx = 0;
            for key in row {
                if let Some(blockage) = blockages.get(blockage_idx) {
                    if col_idx == blockage.start_col {
                        if i == blockage.end_row {
                            blockages.remove(blockage_idx);
                        } else {
                            col_idx = blockage.end_col;
                            blockage_idx += 1;
                        }
                    }
                }
                let tile = &self.tiles[*key];
                if tile.rowspan > 1 {
                    blockages.push(ColBlockage {
                        start_col: col_idx,
                        end_col: col_idx + tile.colspan,
                        end_row: i + tile.rowspan,
                    });
                }
                indices.push((i, col_idx, *key));
                col_idx += tile.colspan;
            }
        }

        indices
    }

    /// Aligns the inserted tiles in a [`TiledGrid`].
    pub fn tile(mut self) -> TiledGrid<PDK> {
        let mut row_constraints = GridConstraintSolver::new();
        let mut col_constraints = GridConstraintSolver::new();

        let indices = self.calculate_indices();

        for (i, j, key) in indices.iter().cloned() {
            let tile = &self.tiles[key];

            if let Some(raw) = &tile.raw {
                row_constraints.add(GridConstraint::new(i, i + tile.rowspan, raw.rect.height()));
                col_constraints.add(GridConstraint::new(j, j + tile.colspan, raw.rect.width()));
            }
        }

        let row_grid = row_constraints.solve();
        let col_grid = col_constraints.solve();

        for (i, j, key) in indices.iter().cloned() {
            let tile = &mut self.tiles[key];

            if let Some(raw) = &mut tile.raw {
                let align_rect = Rect::from_sides(
                    col_grid[j],
                    -row_grid[i + tile.rowspan],
                    col_grid[j + tile.colspan],
                    -row_grid[i],
                );

                raw.align_mut(AlignMode::Top, raw.rect, align_rect, 0);
                raw.align_mut(AlignMode::Left, raw.rect, align_rect, 0);
            }
        }

        TiledGrid { tiles: self.tiles }
    }
}

impl<PDK: Pdk, T: Tileable<PDK>> std::ops::Index<GridTileKey<T>> for TiledGrid<PDK> {
    type Output = T;

    fn index(&self, index: GridTileKey<T>) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<PDK: Pdk> TiledGrid<PDK> {
    /// Gets a tiled object using its [`GridTileKey`].
    pub fn get<T: Tileable<PDK>>(&self, key: GridTileKey<T>) -> Option<&T> {
        self.tiles
            .get(key.key)
            .and_then(|raw| raw.raw.as_ref())
            .and_then(|raw| raw.inner.as_ref().downcast_ref())
    }
}

impl<PDK: Pdk> Draw<PDK> for TiledGrid<PDK> {
    fn draw(self, cell: &mut DrawReceiver<PDK>) -> crate::error::Result<()> {
        for (_, tile) in self.tiles {
            if let Some(raw) = tile.raw {
                raw.draw(cell)?;
            }
        }
        Ok(())
    }
}
