use std::{any::Any, marker::PhantomData, sync::Arc};

use downcast_rs::{impl_downcast, Downcast};
use slotmap::{new_key_type, SlotMap};
use substrate::geometry::dims::Dims;

/// A resizable instance.
pub trait ResizableInstance {
    /// The tile that this is an instance of.
    type Tile: Any;

    /// Increments of the width and height of this tile.
    fn wh_increments(&self) -> Dims;

    /// Parametrization of this tile with the desired strength, width, and height;
    ///
    /// Panics if dimensions are invalid.
    fn tile(&self, dims: Dims) -> Self::Tile;

    /// The maximum width that [`ResizeableTile::min_width`] can return.
    fn max_min_width(&self) -> i64;

    /// Minimum height of this tile required to achieve the desired strength
    /// while fitting in the given width constraint.
    ///
    /// Returns `None` if no parametrization with the desired strength
    /// fits into the given width constraint.
    fn min_height(&self, w_max: i64) -> Option<i64>;

    /// Minimum width of this tile required to achieve the desired strength
    /// while fitting in the given height constraint.
    ///
    /// Returns `None` if no parametrization with the desired strength
    /// fits into the given height constraint.
    fn min_width(&self, h_max: i64) -> Option<i64> {
        // lo is exclusive
        let mut lo = 0;
        let mut hi = self.max_min_width();
        while lo + 1 < hi {
            let mid = (lo + hi) / 2;
            let h = self.min_height(mid);
            if let Some(h) = h {
                if h > h_max {
                    lo = mid;
                } else {
                    hi = mid
                }
            } else {
                lo = mid;
            }
        }
        self.min_height(hi)
            .and_then(|h_min| (h_min <= h_max).then_some(hi))
    }
}

trait DowncastableResizableInstance: Downcast {
    /// Increments of the width and height of this tile.
    fn wh_increments(&self) -> Dims;

    /// Parametrization of this tile with the desired strength, width, and height;
    fn tile(&self, dims: Dims) -> Arc<dyn Any>;

    /// The maximum width that [`ResizeableTile::min_width`] can return.
    fn max_min_width(&self) -> i64;

    /// Minimum height of this tile required to achieve the desired strength
    /// while fitting in the given width constraint.
    ///
    /// Returns `None` if no parametrization with the desired strength
    /// fits into the given width constraint.
    fn min_height(&self, w_max: i64) -> Option<i64>;

    /// Minimum width of this tile required to achieve the desired strength
    /// while fitting in the given height constraint.
    ///
    /// Returns `None` if no parametrization with the desired strength
    /// fits into the given height constraint.
    fn min_width(&self, h_max: i64) -> Option<i64>;
}

impl<T: ResizableInstance + Any> DowncastableResizableInstance for T {
    fn wh_increments(&self) -> Dims {
        ResizableInstance::wh_increments(self)
    }

    fn tile(&self, dims: Dims) -> Arc<dyn Any> {
        Arc::new(ResizableInstance::tile(self, dims))
    }

    fn max_min_width(&self) -> i64 {
        ResizableInstance::max_min_width(self)
    }

    fn min_height(&self, w_max: i64) -> Option<i64> {
        ResizableInstance::min_height(self, w_max)
    }

    fn min_width(&self, h_max: i64) -> Option<i64> {
        ResizableInstance::min_width(self, h_max)
    }
}

impl_downcast!(DowncastableResizableInstance);

struct RawGridTile {
    inst: Arc<dyn DowncastableResizableInstance>,
    rowspan: usize,
}

impl RawGridTile {
    fn new<T: ResizableInstance + Any>(inst: T, row_span: usize) -> Self {
        assert!(row_span > 0);
        Self {
            inst: Arc::new(inst),
            row_span,
        }
    }
}

impl<T: ResizableInstance + Any> From<GridTile<T>> for RawGridTile {
    fn from(GridTile { inst, row_span }: GridTile<T>) -> Self {
        Self::new(inst, row_span)
    }
}

new_key_type! {
    struct RawGridTileKey;
}

struct GridTile<T> {
    inst: T,
    row_span: usize,
}

impl<T> GridTile<T> {
    pub fn new(inst: T, row_span: usize) -> Self {
        assert!(row_span > 0);
        Self { inst, row_span }
    }
}

/// A key for indexing a [`GridTile`] within an [`GridTiler`].
pub struct GridTileKey<T> {
    key: RawGridTileKey,
    phantom: PhantomData<T>,
}

pub struct ResizableGrid {
    wh_increments: Dims,
    tiles: SlotMap<RawGridTileKey, RawGridTile>,
    columns: Vec<Vec<RawGridTileKey>>,
    transpose: bool,
}

pub struct SizedGrid {}

impl Default for ResizableGrid {
    fn default() -> Self {
        Self {
            wh_increments: Dims::new(1, 1),
            tiles: SlotMap::with_key(),
            columns: vec![vec![]],
            transpose: false,
        }
    }
}

impl ResizableGrid {
    /// Creates a new [`ResizeableGrid`].
    pub fn new() -> Self {
        Self::default()
    }

    pub fn transpose(&mut self) {
        self.transpose = true;
    }

    fn last_col_mut(&mut self) -> &mut Vec<RawGridTileKey> {
        self.columns.last_mut().unwrap()
    }

    pub fn push_tile<T: ResizableInstance + Any>(&mut self, tile: GridTile<T>) -> GridTileKey<T> {
        let raw_tile: RawGridTile = tile.into();
        let key = self.tiles.insert(raw_tile);
        self.last_col_mut().push(key);
        GridTileKey {
            key,
            phantom: PhantomData,
        }
    }

    pub fn end_column(&mut self) {
        self.columns.push(Vec::new());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl ResizableInstance for i64 {
        type Tile = ();

        fn wh_increments(&self) -> Dims {
            Dims::new(1, 1)
        }

        fn tile(&self, dims: Dims) -> Self::Tile {}

        fn min_height(&self, w_max: i64) -> i64 {
            std::cmp::max(50 - w_max, 1)
        }
    }

    #[test]
    fn grid_test() {
        let tile = GridTile::new(5, 2);
    }
}
