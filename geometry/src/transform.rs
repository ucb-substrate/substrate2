//! Transformation types and traits.

use serde::{Deserialize, Serialize};

use super::orientation::Orientation;
use crate::point::Point;
use crate::wrap_angle;

/// A transformation representing translation, rotation, and reflection of geometry.
///
/// This object does not support scaling of geometry.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Transformation {
    /// The transformation matrix represented in row-major order.
    pub(crate) a: [[f64; 2]; 2],
    /// The x-y translation applied after the transformation.
    pub(crate) b: [f64; 2],
}
impl Transformation {
    /// Returns the identity transform, leaving any transformed object unmodified.
    pub fn identity() -> Self {
        Self {
            a: [[1., 0.], [0., 1.]],
            b: [0., 0.],
        }
    }
    /// Returns a translation by `(x,y)`.
    pub fn translate(x: f64, y: f64) -> Self {
        Self {
            a: [[1., 0.], [0., 1.]],
            b: [x, y],
        }
    }
    /// Returns a rotatation by `angle` degrees.
    pub fn rotate(angle: f64) -> Self {
        let sin = angle.to_radians().sin();
        let cos = angle.to_radians().cos();
        Self {
            a: [[cos, -sin], [sin, cos]],
            b: [0., 0.],
        }
    }
    /// Returns a reflection about the x-axis.
    pub fn reflect_vert() -> Self {
        Self {
            a: [[1., 0.], [0., -1.]],
            b: [0., 0.],
        }
    }

    /// Returns a new [`TransformationBuilder`].
    #[inline]
    pub fn builder() -> TransformationBuilder {
        TransformationBuilder::default()
    }

    /// Creates a transform from only an offset.
    ///
    /// The resulting transformation will apply only a translation
    /// (i.e. no rotations/reflections).
    pub fn from_offset(offset: Point) -> Self {
        Self::builder()
            .point(offset)
            .orientation(Orientation::default())
            .build()
    }

    /// Creates a transform from an offset and [`Orientation`].
    pub fn from_offset_and_orientation(offset: Point, orientation: impl Into<Orientation>) -> Self {
        Self::builder()
            .point(offset)
            .orientation(orientation.into())
            .build()
    }

    /// Creates a transform from an offset, angle, and a bool indicating
    /// whether or not to reflect vertically.
    pub fn from_opts(offset: Point, reflect_vert: bool, angle: f64) -> Self {
        Self::builder()
            .point(offset)
            .reflect_vert(reflect_vert)
            .angle(angle)
            .build()
    }

    /// Create a new [`Transformation`] that is the cascade of `parent` and `child`.
    ///
    /// "Parents" and "children" refer to typical layout-instance hierarchies,
    /// in which each layer of instance has a nested set of transformations relative to its top-level parent.
    ///
    /// Note this operation *is not* commutative.
    /// For example the set of transformations:
    /// * (a) Reflect vertically, then
    /// * (b) Translate by (1,1)
    /// * (c) Place a point at (local coordinate) (1,1)
    /// Lands said point at (2,-2) in top-level space,
    /// whereas reversing the order of (a) and (b) lands it at (2,0).
    pub fn cascade(parent: Transformation, child: Transformation) -> Transformation {
        // The result-transform's origin is the parent's origin,
        // plus the parent-transformed child's origin
        let mut b = matvec(&parent.a, &child.b);
        b[0] += parent.b[0];
        b[1] += parent.b[1];
        // And the cascade-matrix is the product of the parent's and child's
        let a = matmul(&parent.a, &child.a);
        Self { a, b }
    }

    /// The point representing the translation of this transformation.
    pub fn offset_point(&self) -> Point {
        Point {
            x: self.b[0].round() as i64,
            y: self.b[1].round() as i64,
        }
    }

    /// Returns an [`Orientation`] corresponding to this transformation.
    pub fn orientation(&self) -> Orientation {
        let reflect_vert = self.a[0][0].signum() != self.a[1][1].signum();
        let sin = self.a[1][0];
        let cos = self.a[0][0];
        let angle = cos.acos().to_degrees();
        let angle = if sin > 0f64 {
            angle
        } else {
            wrap_angle(-angle)
        };
        Orientation {
            reflect_vert,
            angle,
        }
    }
}

impl<T> From<T> for Transformation
where
    T: Into<Orientation>,
{
    fn from(value: T) -> Self {
        Self::builder().orientation(value).build()
    }
}

/// A builder for creating transformations from translations and [`Orientation`]s.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TransformationBuilder {
    x: f64,
    y: f64,
    reflect_vert: bool,
    angle: f64,
}

impl TransformationBuilder {
    /// Specifies the x-y translation encoded by the transformation.
    pub fn point(&mut self, point: impl Into<Point>) -> &mut Self {
        let point = point.into();
        self.x = point.x as f64;
        self.y = point.y as f64;
        self
    }

    /// Specifies the [`Orientation`] applied by this transformation.
    pub fn orientation(&mut self, o: impl Into<Orientation>) -> &mut Self {
        let o = o.into();
        self.reflect_vert = o.reflect_vert;
        self.angle = o.angle;
        self
    }

    /// Specifies the angle of rotation encoded by this transformation.
    pub fn angle(&mut self, angle: f64) -> &mut Self {
        self.angle = angle;
        self
    }

    /// Specifies whether the transformation results in a vertical reflection.
    pub fn reflect_vert(&mut self, reflect_vert: bool) -> &mut Self {
        self.reflect_vert = reflect_vert;
        self
    }

    /// Builds a [`Transformation`] from the specified parameters.
    pub fn build(&mut self) -> Transformation {
        let b = [self.x, self.y];
        let sin = self.angle.to_radians().sin();
        let cos = self.angle.to_radians().cos();
        let sin_refl = if self.reflect_vert { sin } else { -sin };
        let cos_refl = if self.reflect_vert { -cos } else { cos };
        let a = [[cos, sin_refl], [sin, cos_refl]];
        Transformation { a, b }
    }
}

/// Multiples two 2x2 matrices, returning a new 2x2 matrix
fn matmul(a: &[[f64; 2]; 2], b: &[[f64; 2]; 2]) -> [[f64; 2]; 2] {
    [
        [
            a[0][0] * b[0][0] + a[0][1] * b[1][0],
            a[0][0] * b[0][1] + a[0][1] * b[1][1],
        ],
        [
            a[1][0] * b[0][0] + a[1][1] * b[1][0],
            a[1][0] * b[0][1] + a[1][1] * b[1][1],
        ],
    ]
}
/// Multiplies a 2x2 matrix by a 2-entry vector, returning a new 2-entry vector.
fn matvec(a: &[[f64; 2]; 2], b: &[f64; 2]) -> [f64; 2] {
    [
        a[0][0] * b[0] + a[0][1] * b[1],
        a[1][0] * b[0] + a[1][1] * b[1],
    ]
}

/// A trait for specifying how an object is changed by a transformation.
pub trait Transform {
    /// Applies matrix-vector [`Transformation`] `trans`.
    fn transform(&mut self, trans: Transformation) -> &mut Self;
}

macro_rules! transform_tuple_impls {
    ( $( ( $name:ident, $idx:tt ) )+ ) => {
        impl<$($name: Transform),+> Transform for ($($name,)+)
        {
            fn transform(&mut self, trans: Transformation) -> &mut Self {
                $(self.$idx.transform(trans);)+
                self
            }
        }
    };
}

transform_tuple_impls! { (A, 0) }
transform_tuple_impls! { (A, 0) (B, 1) }
transform_tuple_impls! { (A, 0) (B, 1) (C, 2) }
transform_tuple_impls! { (A, 0) (B, 1) (C, 2) (D, 3) }

impl<T: Transform> Transform for Vec<T> {
    fn transform(&mut self, trans: Transformation) -> &mut Self {
        for i in self.iter_mut() {
            i.transform(trans);
        }
        self
    }
}

/// A trait for specifying how an object is changed by a transformation.
///
/// Takes in an owned copy of the shape and returns the transformed version.
pub trait TransformOwned: Transform + Sized {
    /// Applies matrix-vector [`Transformation`] `trans`.
    ///
    /// Creates a new shape at a location equal to the transformation of our own.
    fn transform_owned(mut self, trans: Transformation) -> Self {
        self.transform(trans);
        self
    }
}

impl<T: Transform + Sized> TransformOwned for T {}

/// A trait for specifying how a shape is translated by a [`Point`].
pub trait Translate {
    /// Translates the shape by a [`Point`] through mutation.
    fn translate(&mut self, p: Point);
}

macro_rules! translate_tuple_impls {
    ( $( ( $name:ident, $idx:tt ) )+ ) => {
        impl<$($name: Translate),+> Translate for ($($name,)+)
        {
            fn translate(&mut self, p: Point) {
                $(self.$idx.translate(p);)+
            }
        }
    };
}

translate_tuple_impls! { (A, 0) }
translate_tuple_impls! { (A, 0) (B, 1) }
translate_tuple_impls! { (A, 0) (B, 1) (C, 2) }
translate_tuple_impls! { (A, 0) (B, 1) (C, 2) (D, 3) }

impl<T: Translate> Translate for Vec<T> {
    fn translate(&mut self, p: Point) {
        for i in self.iter_mut() {
            i.translate(p);
        }
    }
}

/// A trait for specifying how a shape is translated by a [`Point`].
///
/// Takes in an owned copy of the shape and returns the translated version.
pub trait TranslateOwned: Translate + Sized {
    /// Translates the shape by a [`Point`] through mutation.
    ///
    /// Creates a new shape at a location equal to the translation of our own.
    fn translate_owned(mut self, p: Point) -> Self {
        self.translate(p);
        self
    }
}

impl<T: Translate + Sized> TranslateOwned for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orientation::NamedOrientation;

    #[test]
    fn matvec_works() {
        let a = [[1., 2.], [3., 4.]];
        let b = [5., 6.];
        assert_eq!(matvec(&a, &b), [17., 39.]);
    }

    #[test]
    fn matmul_works() {
        let a = [[1., 2.], [3., 4.]];
        let b = [[5., 6.], [7., 8.]];
        assert_eq!(matmul(&a, &b), [[19., 22.], [43., 50.]]);
    }

    #[test]
    fn cascade_identity_preserves_transformation() {
        for orientation in NamedOrientation::all_rectangular() {
            let tf = Transformation::from_offset_and_orientation(Point::new(520, 130), orientation);
            let casc = Transformation::cascade(tf, Transformation::identity());
            assert_eq!(
                tf, casc,
                "Cascading with identity produced incorrect transformation for orientation {:?}",
                orientation
            );
        }
    }

    #[test]
    fn transformation_offset_and_orientation_preserves_components() {
        let pt = Point::new(8930, 730);
        for orientation in NamedOrientation::all_rectangular() {
            println!("Testing orientation {:?}", orientation);
            let tf = Transformation::from_offset_and_orientation(pt, orientation);
            assert_eq!(tf.orientation(), orientation.into());
            assert_eq!(tf.offset_point(), pt);
        }
    }

    #[test]
    fn transformation_equivalent_to_offset_and_orientation() {
        for orientation in NamedOrientation::all_rectangular() {
            println!("Testing orientation {:?}", orientation);
            let tf1 =
                Transformation::from_offset_and_orientation(Point::new(380, 340), orientation);
            assert_eq!(tf1.orientation(), orientation.into());
            let tf2 =
                Transformation::from_offset_and_orientation(tf1.offset_point(), tf1.orientation());
            assert_eq!(tf1, tf2);
        }
    }

    #[test]
    fn point_transformations_work() {
        let pt = Point::new(2, 1);

        let pt_reflect_vert = pt.transform_owned(Transformation::from_offset_and_orientation(
            Point::zero(),
            NamedOrientation::ReflectVert,
        ));
        assert_eq!(pt_reflect_vert, Point::new(2, -1));

        let pt_reflect_horiz = pt.transform_owned(Transformation::from_offset_and_orientation(
            Point::zero(),
            NamedOrientation::ReflectHoriz,
        ));
        assert_eq!(pt_reflect_horiz, Point::new(-2, 1));

        let pt_r90 = pt.transform_owned(Transformation::from_offset_and_orientation(
            Point::new(23, 11),
            NamedOrientation::R90,
        ));
        assert_eq!(pt_r90, Point::new(22, 13));

        let pt_r180 = pt.transform_owned(Transformation::from_offset_and_orientation(
            Point::new(-50, 10),
            NamedOrientation::R180,
        ));
        assert_eq!(pt_r180, Point::new(-52, 9));

        let pt_r270 = pt.transform_owned(Transformation::from_offset_and_orientation(
            Point::new(80, 90),
            NamedOrientation::R270,
        ));
        assert_eq!(pt_r270, Point::new(81, 88));

        let pt_r90cw = pt.transform_owned(Transformation::from_offset_and_orientation(
            Point::new(5, 13),
            NamedOrientation::R90Cw,
        ));
        assert_eq!(pt_r90cw, Point::new(6, 11));

        let pt_r180cw = pt.transform_owned(Transformation::from_offset_and_orientation(
            Point::zero(),
            NamedOrientation::R180Cw,
        ));
        assert_eq!(pt_r180cw, Point::new(-2, -1));

        let pt_r270cw = pt.transform_owned(Transformation::from_offset_and_orientation(
            Point::new(1, 100),
            NamedOrientation::R270Cw,
        ));
        assert_eq!(pt_r270cw, Point::new(0, 102));

        let pt_flip_yx = pt.transform_owned(Transformation::from_offset_and_orientation(
            Point::new(-65, -101),
            NamedOrientation::FlipYx,
        ));
        assert_eq!(pt_flip_yx, Point::new(-64, -99));

        let pt_flip_minus_yx = pt.transform_owned(Transformation::from_offset_and_orientation(
            Point::new(1, -5),
            NamedOrientation::FlipMinusYx,
        ));
        assert_eq!(pt_flip_minus_yx, Point::new(0, -7));
    }
}
