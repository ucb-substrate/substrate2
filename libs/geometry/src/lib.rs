//! 2-D geometric operations relevant to integrated circuit layout.
//!
//! # Examples
//!
//! Create a [rectangle](crate::rect::Rect):
//!
//! ```
//! # use geometry::prelude::*;
//! let rect = Rect::from_sides(10, 20, 30, 40);
//! ```
#![warn(missing_docs)]

extern crate self as geometry;

pub mod align;
pub mod bbox;
pub mod contains;
pub mod corner;
pub mod dims;
pub mod dir;
pub mod edge;
pub mod intersect;
pub mod orientation;
pub mod place;
pub mod point;
pub mod polygon;
pub mod prelude;
pub mod rect;
pub mod ring;
pub mod shape;
pub mod side;
pub mod sign;
pub mod snap;
pub mod span;
pub mod transform;
pub mod union;

/// Wraps the given angle to the interval `[0, 360)` degrees.
///
/// # Examples
///
/// ```
/// use geometry::wrap_angle;
///
/// assert_eq!(wrap_angle(10.), 10.);
/// assert_eq!(wrap_angle(-10.), 350.);
/// assert_eq!(wrap_angle(-740.), 340.);
/// assert_eq!(wrap_angle(-359.), 1.);
/// assert_eq!(wrap_angle(-1.), 359.);
/// assert_eq!(wrap_angle(725.), 5.);
/// assert_eq!(wrap_angle(360.), 0.);
/// assert_eq!(wrap_angle(-360.), 0.);
/// ```
pub fn wrap_angle(angle: f64) -> f64 {
    ((angle % 360.) + 360.) % 360.
}
