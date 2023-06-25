//! Generic layout cells and instances.

use std::sync::Arc;

use geometry::{
    prelude::{Bbox, Orientation, Point},
    transform::{Transform, TransformMut, Transformation, TranslateMut},
};
use once_cell::sync::OnceCell;

use super::{
    draw::Draw,
    element::{RawCell, RawInstance},
    HasLayout,
};
use crate::error::Result;

/// A generic layout cell.
///
/// Stores its underlying block, extra data created during generation, as well as a raw cell
/// containing its primitive elements.
///
/// # Examples
///
/// ```
#[doc = include_str!("../../../docs/api/code/prelude.md.hidden")]
#[doc = include_str!("../../../docs/api/code/pdk/pdk.md.hidden")]
#[doc = include_str!("../../../docs/api/code/block/inverter.md.hidden")]
#[doc = include_str!("../../../docs/api/code/layout/inverter.md.hidden")]
#[doc = include_str!("../../../docs/api/code/block/buffer.md.hidden")]
#[doc = include_str!("../../../docs/api/code/layout/buffer.md.hidden")]
#[doc = include_str!("../../../docs/api/code/layout/generate.md")]
/// ```
#[derive(Default, Clone)]
#[allow(dead_code)]
pub struct Cell<T: HasLayout> {
    /// Block whose layout this cell represents.
    pub block: T,
    /// Extra data created during layout generation.
    pub data: T::Data,
    pub(crate) raw: Arc<RawCell>,
}

impl<T: HasLayout> Cell<T> {
    pub(crate) fn new(block: T, data: T::Data, raw: Arc<RawCell>) -> Self {
        Self { block, data, raw }
    }
}

impl<T: HasLayout> Bbox for Cell<T> {
    fn bbox(&self) -> Option<geometry::rect::Rect> {
        self.raw.bbox()
    }
}

/// A generic layout instance.
///
/// Stores a pointer to its underlying cell and its instantiated location and orientation.
#[derive(Clone)]
#[allow(dead_code)]
pub struct Instance<T: HasLayout> {
    cell: Arc<OnceCell<Result<Cell<T>>>>,
    pub(crate) loc: Point,
    pub(crate) orientation: Orientation,
}

impl<T: HasLayout> Instance<T> {
    pub(crate) fn new(cell: Arc<OnceCell<Result<Cell<T>>>>) -> Self {
        Instance {
            cell,
            loc: Point::default(),
            orientation: Orientation::default(),
        }
    }

    /// Tries to access the underlying [`Cell`].
    ///
    /// Returns an error if one was thrown during generation.
    pub fn try_cell(&self) -> &Result<Cell<T>> {
        self.cell.wait()
    }

    /// Returns the underlying [`Cell`].
    ///
    /// # Panics
    ///
    /// Panics if an error was thrown during generation.
    pub fn cell(&self) -> &Cell<T> {
        self.try_cell().as_ref().unwrap()
    }

    /// Returns the current transformation of `self`.
    pub fn transformation(&self) -> Transformation {
        Transformation::from_offset_and_orientation(self.loc, self.orientation)
    }
}

impl<T: HasLayout> Bbox for Instance<T> {
    fn bbox(&self) -> Option<geometry::rect::Rect> {
        self.cell()
            .bbox()
            .map(|rect| rect.transform(self.transformation()))
    }
}

impl<T: HasLayout> TranslateMut for Instance<T> {
    fn translate_mut(&mut self, p: Point) {
        self.loc.translate_mut(p);
    }
}

impl<T: HasLayout> TransformMut for Instance<T> {
    fn transform_mut(&mut self, trans: Transformation) {
        let new_transform = Transformation::cascade(self.transformation(), trans);
        self.loc = new_transform.offset_point();
        self.orientation = new_transform.orientation();
    }
}

impl<I: HasLayout> Draw for Instance<I> {
    fn draw<T: super::draw::DrawContainer + ?Sized>(self, container: &mut T) {
        RawInstance::from(self).draw(container);
    }
}
