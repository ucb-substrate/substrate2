//! Utilities and types for orienting layout objects.

use std::hash::Hash;

use serde::{Deserialize, Serialize};

use crate::transform::Transformation;

/// A named orientation.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Named {
    /// No rotations or reflections.
    #[default]
    Default,
    /// Reflect vertically (ie. about the x-axis).
    ReflectVert,
    /// Reflect horizontally (ie. about the y-axis).
    ReflectHoriz,
    /// Rotate 90 degrees counter-clockwise.
    R90,
    /// Rotate 180 degrees counter-clockwise.
    R180,
    /// Rotate 270 degrees counter-clockwise.
    R270,
    /// Rotate 90 degrees clockwise.
    R90Cw,
    /// Rotate 180 degrees clockwise.
    R180Cw,
    /// Rotate 270 degrees clockwise.
    R270Cw,
    /// Flip across the line y = x.
    FlipYx,
    /// Flip across the line y = -x.
    FlipMinusYx,
}

/// An orientation of a cell instance.
#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Orientation {
    /// Reflect vertically.
    ///
    /// Applied before rotation.
    pub(crate) reflect_vert: bool,
    /// Counter-clockwise angle in degrees.
    ///
    /// Applied after reflecting vertically.
    pub(crate) angle: f64,
}

/// An orientation of a cell instance, represented as raw bytes.
#[derive(Debug, Default, Copy, Clone, Eq, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct OrientationBytes {
    /// Reflect vertically.
    ///
    /// Applied before rotation.
    pub(crate) reflect_vert: bool,
    /// Counter-clockwise angle in degrees.
    ///
    /// Applied after reflecting vertically.
    pub(crate) angle: u64,
}

impl From<Orientation> for OrientationBytes {
    fn from(value: Orientation) -> Self {
        Self {
            reflect_vert: value.reflect_vert,
            angle: value.angle.to_bits(),
        }
    }
}

impl Named {
    /// Returns a slice of all 8 possible named rectangular orientations.
    pub fn all_rectangular() -> [Self; 8] {
        [
            Self::Default,
            Self::ReflectVert,
            Self::ReflectHoriz,
            Self::R90,
            Self::R180,
            Self::R270,
            Self::FlipYx,
            Self::FlipMinusYx,
        ]
    }

    /// Converts this named orientation into a regular [`Orientation`].
    #[inline]
    pub fn into_orientation(self) -> Orientation {
        Orientation::from(self)
    }
}

impl From<Named> for Orientation {
    fn from(value: Named) -> Self {
        use Named::*;
        let (reflect_vert, angle) = match value {
            Default => (false, 0.),
            R90 | R270Cw => (false, 90.),
            R180 | R180Cw => (false, 180.),
            R270 | R90Cw => (false, 270.),
            ReflectVert => (true, 0.),
            FlipYx => (true, 90.),
            ReflectHoriz => (true, 180.),
            FlipMinusYx => (true, 270.),
        };
        Self {
            reflect_vert,
            angle,
        }
    }
}

impl Orientation {
    /// Returns the identity orientation with `reflect_vert = false` and `angle = 0.`.
    pub fn identity() -> Self {
        Self::default()
    }

    /// Applies the reflection and rotation specified in
    /// [`Orientation`] `o` to this orientation.
    pub fn apply(&mut self, o: impl Into<Orientation>) {
        let o = o.into();
        match (self.reflect_vert, o.reflect_vert) {
            (false, false) => {
                self.angle += o.angle;
            }
            (false, true) => {
                self.reflect_vert = true;
                self.angle = o.angle - self.angle;
            }
            (true, false) => {
                self.angle += o.angle;
            }
            (true, true) => {
                self.reflect_vert = false;
                self.angle = o.angle - self.angle;
            }
        }

        // Keep the angle between 0 and 360 degrees.
        self.wrap_angle();
    }

    /// Reflects the orientation vertically.
    #[inline]
    pub fn reflect_vert(&mut self) {
        self.apply(Named::ReflectVert);
    }

    /// Reflects the orientation horizontally.
    #[inline]
    pub fn reflect_horiz(&mut self) {
        self.apply(Named::ReflectHoriz);
    }

    /// Rotates the orientation 90 degrees counter-clockwise.
    #[inline]
    pub fn r90(&mut self) {
        self.apply(Named::R90);
    }

    /// Rotates the orientation 180 degrees.
    #[inline]
    pub fn r180(&mut self) {
        self.apply(Named::R180);
    }

    /// Rotates the orientation 180 degrees counter-clockwise.
    #[inline]
    pub fn r270(&mut self) {
        self.apply(Named::R270);
    }

    /// Rotates the orientation 90 degrees clockwise.
    #[inline]
    pub fn r90cw(&mut self) {
        self.apply(Named::R90Cw);
    }

    /// Rotates the orientation 180 degrees clockwise.
    pub fn r180cw(&mut self) {
        self.apply(Named::R180Cw);
    }

    /// Rotates the orientation 270 degrees clockwise.
    #[inline]
    pub fn r270cw(&mut self) {
        self.apply(Named::R270Cw);
    }

    /// Flips the orientation around the line `y = x`.
    #[inline]
    pub fn flip_yx(&mut self) {
        self.apply(Named::FlipYx);
    }

    /// Flips the orientation around the line `y = -x`.
    #[inline]
    pub fn flip_minus_yx(&mut self) {
        self.apply(Named::FlipMinusYx);
    }

    /// Returns the angle associated with this orientation.
    #[inline]
    pub fn angle(&self) -> f64 {
        self.angle
    }

    /// Wraps the given angle to the interval `[0, 360)` degrees.
    #[inline]
    fn wrap_angle(&mut self) {
        self.angle = wrap_angle(self.angle);
    }

    #[inline]
    pub fn from_transformation(value: Transformation) -> Self {
        value.orientation()
    }

    /// Returns a slice of all 8 possible rectangular orientations.
    pub fn all_rectangular() -> [Self; 8] {
        Named::all_rectangular().map(Self::from)
    }
}

/// Wraps the given angle to the interval `[0, 360)` degrees.
///
/// # Examples
///
/// ```
/// use geometry::orientation::wrap_angle;
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_named_all_rectangular() {
        let opts: [OrientationBytes; 8] = Named::all_rectangular()
            .map(|n| n.into_orientation())
            .map(OrientationBytes::from);
        let mut set = HashSet::new();
        for item in opts {
            set.insert(item);
        }
        assert_eq!(set.len(), 8);
    }
}
