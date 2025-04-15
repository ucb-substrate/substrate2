//! Utilities for resizing tiles given width or height constraints.

use std::{any::Any, marker::PhantomData, sync::Arc};

use downcast_rs::{impl_downcast, Downcast};
use slotmap::{new_key_type, SecondaryMap, SlotMap};
use substrate::geometry::{dims::Dims, point::Point, rect::Rect, transform::Translate};

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

    /// The maximum width that [`ResizableGrid::min_width`] can return.
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

    fn min_height(&self, w_max: i64) -> Option<i64> {
        ResizableInstance::min_height(self, w_max)
    }

    fn min_width(&self, h_max: i64) -> Option<i64> {
        ResizableInstance::min_width(self, h_max)
    }
}

impl_downcast!(DowncastableResizableInstance);

type RawInst = Arc<dyn DowncastableResizableInstance>;

new_key_type! {
    struct RawInstKey;
}

/// A key for indexing a instance within an [`ResizableGrid`].
pub struct InstKey<T> {
    key: RawInstKey,
    phantom: PhantomData<T>,
}

/// A resizable grid.
pub struct ResizableGrid {
    tiles: SlotMap<RawInstKey, RawInst>,
    columns: Vec<Vec<RawInstKey>>,
    transpose: bool,
}

impl Default for ResizableGrid {
    fn default() -> Self {
        Self {
            tiles: SlotMap::with_key(),
            columns: vec![vec![]],
            transpose: false,
        }
    }
}

impl ResizableGrid {
    /// Creates a new [`ResizableGrid`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Transposes the grid such that it is composed of rows instead of columns.
    pub fn transpose(&mut self) {
        self.transpose = true;
    }

    fn last_col_mut(&mut self) -> &mut Vec<RawInstKey> {
        self.columns.last_mut().unwrap()
    }

    /// Pushes a tile to the current column or row.
    pub fn push_tile<T: ResizableInstance + Any>(&mut self, tile: T) -> InstKey<T> {
        let raw_tile = Arc::new(tile);
        let key = self.tiles.insert(raw_tile);
        self.last_col_mut().push(key);
        InstKey {
            key,
            phantom: PhantomData,
        }
    }

    /// Ends the current column (or row if the grid has been transposed).
    pub fn end_column(&mut self) {
        self.columns.push(Vec::new());
    }

    fn col_min_height(&self, col: usize, w_max: i64) -> Option<i64> {
        self.columns[col]
            .iter()
            .map(|key| {
                let inst = &self.tiles[*key];
                if self.transpose {
                    inst.min_width(w_max)
                } else {
                    inst.min_height(w_max)
                }
            })
            .reduce(|a, b| {
                if let (Some(a), Some(b)) = (a, b) {
                    Some(a + b)
                } else {
                    None
                }
            })
            .unwrap()
    }

    /// Sizes the grid.
    pub fn size(self, w_max: i64) -> SizedGrid {
        // TODO: Merge unconstrained rows.
        let nc = self.columns.len();
        let mut increment = 1;
        for column in &self.columns {
            for key in column {
                let increments = self.tiles[*key].wh_increments();
                increment = num::integer::lcm(
                    increment,
                    if self.transpose {
                        increments.h()
                    } else {
                        increments.w()
                    },
                );
            }
        }
        #[derive(Clone, Debug)]
        struct DpValue {
            min_height: i64,
            widths: Vec<i64>,
        }
        let u_w_max = (w_max / increment) as usize;
        let mut h = vec![vec![None; nc]; u_w_max + 1];
        for w in 0..=u_w_max {
            let w_phys = increment * w as i64;
            for i in 0..nc {
                if i == 0 {
                    h[w][i] = self.col_min_height(i, w_phys).map(|min_height| DpValue {
                        min_height,
                        widths: vec![w_phys],
                    });
                } else {
                    for w_col in 0..w {
                        let w_col_phys = increment * w_col as i64;
                        if let Some(prev_h) = &h[w - w_col][i - 1] {
                            if let Some(h_min) = self.col_min_height(i, w_col_phys) {
                                let tot_height = std::cmp::max(prev_h.min_height, h_min);
                                if h[w][i]
                                    .as_ref()
                                    .map(|curr| tot_height < curr.min_height)
                                    .unwrap_or(true)
                                {
                                    let mut widths = prev_h.widths.clone();
                                    widths.push(w_col_phys);
                                    h[w][i] = Some(DpValue {
                                        min_height: tot_height,
                                        widths,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // Find the smallest width that achieves the same height as the max width.
        let mut best_sizing = h[u_w_max][nc - 1].as_ref().unwrap();
        for i in (0..u_w_max).rev() {
            if let Some(sizing) = &h[i][nc - 1] {
                if sizing.min_height == best_sizing.min_height {
                    best_sizing = sizing;
                }
            } else {
                break;
            }
        }

        // Allocate bboxes.
        let mut bboxes = Vec::new();
        let mut col_heights = Vec::new();
        let mut ll = Point::zero();
        let mut max_col_height = 0;
        for (i, column) in self.columns.iter().enumerate() {
            let mut col_bboxes = Vec::new();
            let w = best_sizing.widths[i];
            for key in column {
                let tile = &self.tiles[*key];
                let (h, dims) = if self.transpose {
                    let h = tile.min_width(w).unwrap();
                    (h, Dims::new(h, w))
                } else {
                    let h = tile.min_height(w).unwrap();
                    (h, Dims::new(w, h))
                };
                col_bboxes.push((*key, Rect::from_dims(dims).translate(ll)));
                if self.transpose {
                    ll.x += h;
                } else {
                    ll.y += h;
                }
            }
            bboxes.push(col_bboxes);
            if self.transpose {
                col_heights.push(ll.x);
                max_col_height = std::cmp::max(max_col_height, ll.x);
                ll.x = 0;
                ll.y += w;
            } else {
                col_heights.push(ll.y);
                max_col_height = std::cmp::max(max_col_height, ll.y);
                ll.y = 0;
                ll.x += w;
            }
        }

        let mut sized_tiles = SecondaryMap::new();
        for (col, col_bboxes) in bboxes.iter().enumerate() {
            for (key, bbox) in col_bboxes {
                let offset = (max_col_height - col_heights[col]) / 2;
                sized_tiles.insert(
                    *key,
                    SizedRawInst {
                        tile: self.tiles[*key].tile(bbox.dims()),
                        phys_bbox: bbox
                            .translate(if self.transpose { (offset, 0) } else { (0, 0) }.into()),
                    },
                );
            }
        }

        SizedGrid {
            tiles: self.tiles,
            sized_tiles,
        }
    }
}

/// A sized grid.
pub struct SizedGrid {
    // Necessary for secondary map to work?
    #[allow(dead_code)]
    tiles: SlotMap<RawInstKey, RawInst>,
    sized_tiles: SecondaryMap<RawInstKey, SizedRawInst>,
}

struct SizedRawInst {
    tile: Arc<dyn Any>,
    phys_bbox: Rect,
}

impl SizedGrid {
    /// Retrieves the sizing and bounding box of a certain resizable tile.
    pub fn get_tile<T: ResizableInstance + Any>(&self, key: InstKey<T>) -> (&T::Tile, Rect) {
        let sized_tile = &self.sized_tiles[key.key];
        (
            sized_tile.tile.downcast_ref().unwrap(),
            sized_tile.phys_bbox,
        )
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

        fn tile(&self, _dims: Dims) -> Self::Tile {}

        fn max_min_width(&self) -> i64 {
            50 * self
        }

        fn min_height(&self, w_max: i64) -> Option<i64> {
            if w_max > 10 {
                Some(std::cmp::max(50 * self - w_max, 1))
            } else {
                None
            }
        }
    }

    #[test]
    fn grid_test() {
        let mut grid = ResizableGrid::new();
        let mut tiles = Vec::new();
        tiles.push(grid.push_tile(5));
        tiles.push(grid.push_tile(10));
        tiles.push(grid.push_tile(20));
        grid.end_column();
        tiles.push(grid.push_tile(40));
        let sized = grid.size(500);
        for tile in tiles {
            println!("{:?}", sized.get_tile(tile));
        }
    }
}
