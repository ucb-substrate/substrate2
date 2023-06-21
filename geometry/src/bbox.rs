//! Axis-aligned rectangular bounding boxes.

use crate::{rect::Rect, union::BboxUnion};

/// A geometric shape that has a bounding box.
///
/// # Examples
///
/// ```
/// # use geometry::prelude::*;
/// let rect = Rect::from_sides(0, 0, 100, 200);
/// assert_eq!(rect.bbox(), Some(Rect::from_sides(0, 0, 100, 200)));
/// let rect = Rect::from_xy(50, 70);
/// assert_eq!(rect.bbox(), Some(Rect::from_sides(50, 70, 50, 70)));
/// ```
pub trait Bbox {
    /// Compute the axis-aligned rectangular bounding box.
    ///
    /// If empty, this method should return `None`.
    /// Note that poinst and zero-area rectangles are not empty:
    /// these shapes contain a single point, and their bounding box
    /// implementations will return `Some(_)`.
    fn bbox(&self) -> Option<Rect>;
}

impl<T> Bbox for &T
where
    T: Bbox,
{
    fn bbox(&self) -> Option<Rect> {
        T::bbox(*self)
    }
}

macro_rules! bbox_tuple_impls {
    ( $( $name:ident )+ ) => {
        #[allow(non_snake_case)]
        impl<$($name: Bbox),+> Bbox for ($($name,)+)
        {
            fn bbox(&self) -> Option<Rect> {
                let ($( $name, )+) = self;
                let mut bbox = None;
                $(bbox = bbox.union(&$name.bbox());)+

                bbox
            }
        }
    };
}

bbox_tuple_impls! { A }
bbox_tuple_impls! { A B }
bbox_tuple_impls! { A B C }
bbox_tuple_impls! { A B C D }
bbox_tuple_impls! { A B C D E }
bbox_tuple_impls! { A B C D E F }
bbox_tuple_impls! { A B C D E F G }
bbox_tuple_impls! { A B C D E F G H }
bbox_tuple_impls! { A B C D E F G H I }
bbox_tuple_impls! { A B C D E F G H I J }
bbox_tuple_impls! { A B C D E F G H I J K }
bbox_tuple_impls! { A B C D E F G H I J K L }

impl<T: Bbox> Bbox for Vec<T> {
    fn bbox(&self) -> Option<Rect> {
        let mut bbox = None;
        for i in self.iter() {
            bbox = bbox.union(&i.bbox());
        }
        bbox
    }
}
