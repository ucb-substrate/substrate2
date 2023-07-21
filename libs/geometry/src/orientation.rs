//! Utilities and types for orienting layout objects.

use std::hash::Hash;

use serde::{Deserialize, Serialize};

use crate::transform::Transformation;

/// A named orientation.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum NamedOrientation {
    /// No rotations or reflections.
    #[default]
    R0,
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

/// An orientation of a geometric object.
///
/// Captures reflection and rotation, but not position or scaling.
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

/// An orientation of a geometric object, represented as raw bytes.
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

impl NamedOrientation {
    /// Returns a slice of all 8 possible named rectangular orientations.
    ///
    /// Users should not rely upon the order of the orientations returned.
    pub fn all_rectangular() -> [Self; 8] {
        [
            Self::R0,
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

impl From<NamedOrientation> for Orientation {
    fn from(value: NamedOrientation) -> Self {
        use NamedOrientation::*;
        let (reflect_vert, angle) = match value {
            R0 => (false, 0.),
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
    /// Creates a new orientation with the given reflection and angle settings.
    #[inline]
    pub fn from_reflect_and_angle(reflect_vert: bool, angle: f64) -> Self {
        Self {
            reflect_vert,
            angle,
        }
    }

    /// Returns the identity orientation with `reflect_vert = false` and `angle = 0.`.
    pub fn identity() -> Self {
        Self::default()
    }

    /// Applies the reflection and rotation specified in
    /// [`Orientation`] `o` to this orientation.
    pub fn apply(mut self, o: impl Into<Orientation>) -> Self {
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
        self.wrap_angle()
    }

    /// Reflects the orientation vertically.
    #[inline]
    pub fn reflected_vert(self) -> Self {
        self.apply(NamedOrientation::ReflectVert)
    }

    /// Reflects the orientation horizontally.
    #[inline]
    pub fn reflected_horiz(self) -> Self {
        self.apply(NamedOrientation::ReflectHoriz)
    }

    /// Rotates the orientation 90 degrees counter-clockwise.
    #[inline]
    pub fn r90(self) -> Self {
        self.apply(NamedOrientation::R90)
    }

    /// Rotates the orientation 180 degrees.
    #[inline]
    pub fn r180(self) -> Self {
        self.apply(NamedOrientation::R180)
    }

    /// Rotates the orientation 180 degrees counter-clockwise.
    #[inline]
    pub fn r270(self) -> Self {
        self.apply(NamedOrientation::R270)
    }

    /// Rotates the orientation 90 degrees clockwise.
    #[inline]
    pub fn r90cw(self) -> Self {
        self.apply(NamedOrientation::R90Cw)
    }

    /// Rotates the orientation 180 degrees clockwise.
    pub fn r180cw(self) -> Self {
        self.apply(NamedOrientation::R180Cw)
    }

    /// Rotates the orientation 270 degrees clockwise.
    #[inline]
    pub fn r270cw(self) -> Self {
        self.apply(NamedOrientation::R270Cw)
    }

    /// Flips the orientation around the line `y = x`.
    #[inline]
    pub fn flip_yx(self) -> Self {
        self.apply(NamedOrientation::FlipYx)
    }

    /// Flips the orientation around the line `y = -x`.
    #[inline]
    pub fn flip_minus_yx(self) -> Self {
        self.apply(NamedOrientation::FlipMinusYx)
    }

    /// Returns whether the orientation is reflected vertically.
    #[inline]
    pub fn reflect_vert(&self) -> bool {
        self.reflect_vert
    }

    /// Returns the angle associated with this orientation.
    #[inline]
    pub fn angle(&self) -> f64 {
        self.angle
    }

    /// Wraps the given angle to the interval `[0, 360)` degrees.
    #[inline]
    fn wrap_angle(mut self) -> Self {
        self.angle = crate::wrap_angle(self.angle);
        self
    }

    /// Returns the orientation represented by the given transformation.
    ///
    /// Captures the rotation and reflection encoded by the [`Transformation`],
    /// discarding the transformation's translation.
    ///
    /// # Example
    ///
    /// ```
    /// # use geometry::prelude::*;
    /// let tf = Transformation::identity();
    /// assert_eq!(Orientation::from_transformation(tf), NamedOrientation::R0.into());
    /// ```
    #[inline]
    pub fn from_transformation(value: Transformation) -> Self {
        value.orientation()
    }

    /// Returns a slice of all 8 possible rectangular orientations.
    ///
    /// Users should not rely upon the order of the orientations returned.
    pub fn all_rectangular() -> [Self; 8] {
        NamedOrientation::all_rectangular().map(Self::from)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn orientations_convert_to_unique_bytes() {
        let opts: [OrientationBytes; 8] = NamedOrientation::all_rectangular()
            .map(|n| n.into_orientation())
            .map(OrientationBytes::from);
        let mut set = HashSet::new();
        for item in opts {
            set.insert(item);
        }
        assert_eq!(set.len(), 8);
    }
}
