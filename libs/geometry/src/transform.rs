//! Transformation types and traits.

use std::f64::consts::PI;

pub use geometry_macros::{TransformMut, TranslateMut};
use impl_trait_for_tuples::impl_for_tuples;
use serde::{Deserialize, Serialize};

use super::orientation::Orientation;
use crate::point::Point;

/// A transformation representing a Manhattan translation, rotation, and/or reflection of geometry.
///
/// This object does not support scaling of geometry, and as such all transformation matrices
/// should be unitary.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transformation {
    /// The transformation matrix.
    pub(crate) mat: TransformationMatrix,
    /// The x-y translation applied after the transformation.
    pub(crate) b: Point,
}

impl Default for Transformation {
    fn default() -> Self {
        Self::identity()
    }
}

/// A Manhattan rotation: 0, 90, 180, or 270 degrees counterclockwise.
#[derive(Debug, Clone, Copy, Default, Eq, Ord, PartialOrd, PartialEq, Serialize, Deserialize)]
pub enum Rotation {
    /// 0 degrees; no rotation.
    #[default]
    R0,
    /// 90 degrees counterclockwise.
    R90,
    /// 180 degrees counterclockwise.
    R180,
    /// 270 degrees counterclockwise.
    R270,
}

impl std::ops::Add<Rotation> for Rotation {
    type Output = Rotation;
    fn add(self, rhs: Rotation) -> Self::Output {
        use Rotation::*;
        match (self, rhs) {
            (R0, R0) => R0,
            (R0, R90) => R90,
            (R0, R180) => R180,
            (R0, R270) => R270,
            (R90, R0) => R90,
            (R90, R90) => R180,
            (R90, R180) => R270,
            (R90, R270) => R0,
            (R180, R0) => R180,
            (R180, R90) => R270,
            (R180, R180) => R0,
            (R180, R270) => R90,
            (R270, R0) => R270,
            (R270, R90) => R0,
            (R270, R180) => R90,
            (R270, R270) => R180,
        }
    }
}

impl std::ops::AddAssign for Rotation {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::Sub<Rotation> for Rotation {
    type Output = Rotation;
    fn sub(self, rhs: Rotation) -> Self::Output {
        use Rotation::*;
        match (self, rhs) {
            (R0, R0) => R0,
            (R0, R90) => R270,
            (R0, R180) => R180,
            (R0, R270) => R90,
            (R90, R0) => R90,
            (R90, R90) => R0,
            (R90, R180) => R270,
            (R90, R270) => R180,
            (R180, R0) => R180,
            (R180, R90) => R90,
            (R180, R180) => R0,
            (R180, R270) => R270,
            (R270, R0) => R270,
            (R270, R90) => R180,
            (R270, R180) => R90,
            (R270, R270) => R0,
        }
    }
}

impl std::ops::SubAssign for Rotation {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Rotation {
    /// The transformation matrix representing this rotation.
    #[inline]
    pub fn transformation_matrix(&self) -> TransformationMatrix {
        TransformationMatrix::from(*self)
    }

    /// The angle of this rotation, in degrees.
    pub fn degrees(&self) -> f64 {
        match self {
            Rotation::R0 => 0.,
            Rotation::R90 => 90.,
            Rotation::R180 => 180.,
            Rotation::R270 => 270.,
        }
    }

    /// The angle of this rotation, in radians.
    pub fn radians(&self) -> f64 {
        match self {
            Rotation::R0 => 0.,
            Rotation::R90 => PI / 2.,
            Rotation::R180 => PI,
            Rotation::R270 => PI * 1.5,
        }
    }
}

/// Indicates that an angle was not a valid Manhattan angle.
///
/// Manhattan angles (in degrees) are 0, 90, 180, 270,
/// or any equivalent angle modulo 360 degrees.
pub struct NonManhattanAngleError;

impl TryFrom<f64> for Rotation {
    type Error = NonManhattanAngleError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        let value = (((value % 360.) + 360.) % 360.).round() as i64;
        match value {
            0 => Ok(Rotation::R0),
            90 => Ok(Rotation::R90),
            180 => Ok(Rotation::R180),
            270 => Ok(Rotation::R270),
            _ => Err(NonManhattanAngleError),
        }
    }
}

/// A matrix representing a unitary transformation.
///
/// Can represent rotations, reflections, or combinations of rotations/reflections.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TransformationMatrix([[i8; 2]; 2]);

impl TransformationMatrix {
    /// The identity transformation.
    ///
    /// Maps any point to itself.
    #[inline]
    pub fn identity() -> Self {
        Self([[1, 0], [0, 1]])
    }

    /// A rotation matrix rotating by the given [`Rotation`].
    pub fn rotate(&self, angle: Rotation) -> Self {
        angle.transformation_matrix() * *self
    }

    /// A matrix representing a reflection across the x-axis.
    pub fn reflect_vert(&self) -> Self {
        Self([[1, 0], [0, -1]]) * *self
    }

    /// The inverse of the transformation matrix.
    pub fn inverse(&self) -> Self {
        Self(unitary_matinv(&self.0))
    }
}

impl From<Rotation> for TransformationMatrix {
    fn from(value: Rotation) -> Self {
        Self(match value {
            Rotation::R0 => [[1, 0], [0, 1]],
            Rotation::R90 => [[0, -1], [1, 0]],
            Rotation::R180 => [[-1, 0], [0, -1]],
            Rotation::R270 => [[0, 1], [-1, 0]],
        })
    }
}

/// Multiples two 2x2 matrices, returning a new 2x2 matrix
fn matmul_i8(a: &[[i8; 2]; 2], b: &[[i8; 2]; 2]) -> [[i8; 2]; 2] {
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
fn matvec_i8_i64(a: &[[i8; 2]; 2], b: &[i64; 2]) -> [i64; 2] {
    [
        a[0][0] as i64 * b[0] + a[0][1] as i64 * b[1],
        a[1][0] as i64 * b[0] + a[1][1] as i64 * b[1],
    ]
}

impl std::ops::Mul<TransformationMatrix> for TransformationMatrix {
    type Output = Self;
    fn mul(self, rhs: TransformationMatrix) -> Self::Output {
        Self(matmul_i8(&self.0, &rhs.0))
    }
}

impl std::ops::Mul<Point> for TransformationMatrix {
    type Output = Point;
    fn mul(self, rhs: Point) -> Self::Output {
        let out = matvec_i8_i64(&self.0, &[rhs.x, rhs.y]);
        Point::new(out[0], out[1])
    }
}

impl std::ops::Deref for TransformationMatrix {
    type Target = [[i8; 2]; 2];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for TransformationMatrix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for TransformationMatrix {
    #[inline]
    fn default() -> Self {
        Self::identity()
    }
}

impl Transformation {
    /// Returns the identity transform, leaving any transformed object unmodified.
    pub fn identity() -> Self {
        Self {
            mat: TransformationMatrix::identity(),
            b: Point::zero(),
        }
    }
    /// Returns a translation by `(x,y)`.
    pub fn translate(x: i64, y: i64) -> Self {
        Self {
            mat: TransformationMatrix::identity(),
            b: Point::new(x, y),
        }
    }
    /// Returns a rotatation by `angle` degrees.
    pub fn rotate(angle: Rotation) -> Self {
        let mat = TransformationMatrix::from(angle);
        Self {
            mat,
            b: Point::zero(),
        }
    }
    /// Returns a reflection about the x-axis.
    pub fn reflect_vert() -> Self {
        Self {
            mat: TransformationMatrix([[1, 0], [0, -1]]),
            b: Point::zero(),
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
    pub fn from_opts(offset: Point, reflect_vert: bool, angle: Rotation) -> Self {
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
    ///
    /// Lands said point at (2,-2) in top-level space,
    /// whereas reversing the order of (a) and (b) lands it at (2,0).
    pub fn cascade(parent: Transformation, child: Transformation) -> Transformation {
        // The result-transform's origin is the parent's origin,
        // plus the parent-transformed child's origin
        let mut b = parent.mat * child.b;
        b += parent.b;
        // And the cascade-matrix is the product of the parent's and child's
        let mat = parent.mat * child.mat;
        Self { mat, b }
    }

    /// The point representing the translation of this transformation.
    pub fn offset_point(&self) -> Point {
        self.b
    }

    /// Returns an [`Orientation`] corresponding to this transformation.
    pub fn orientation(&self) -> Orientation {
        let reflect_vert = self.mat[0][0].signum() != self.mat[1][1].signum();
        let cos = self.mat[0][0];
        let sin = self.mat[1][0];
        let angle = match (cos, sin) {
            (1, 0) => Rotation::R0,
            (0, 1) => Rotation::R90,
            (-1, 0) => Rotation::R180,
            (0, -1) => Rotation::R270,
            _ => panic!("transformation did not represent a valid Manhattan transformation"),
        };
        Orientation {
            reflect_vert,
            angle,
        }
    }

    /// Returns the inverse [`Transformation`] of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use geometry::transform::Transformation;
    /// use approx::assert_relative_eq;
    ///
    /// let trans = Transformation::cascade(
    ///     Transformation::rotate(90),
    ///     Transformation::translate(5., 10),
    /// );
    /// let inv = trans.inv();
    ///
    /// assert_relative_eq!(Transformation::cascade(inv, trans), Transformation::identity());
    /// ```
    pub fn inv(&self) -> Transformation {
        let inv = self.mat.inverse();
        let invb = inv * self.b;
        Self { mat: inv, b: -invb }
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
    x: i64,
    y: i64,
    reflect_vert: bool,
    angle: Rotation,
}

impl TransformationBuilder {
    /// Specifies the x-y translation encoded by the transformation.
    pub fn point(&mut self, point: impl Into<Point>) -> &mut Self {
        let point = point.into();
        self.x = point.x;
        self.y = point.y;
        self
    }

    /// Specifies the [`Orientation`] applied by this transformation.
    ///
    /// This overrides any angle and reflection settings previously applied.
    pub fn orientation(&mut self, o: impl Into<Orientation>) -> &mut Self {
        let o = o.into();
        self.reflect_vert = o.reflect_vert;
        self.angle = o.angle;
        self
    }

    /// Specifies the angle of rotation encoded by this transformation.
    pub fn angle(&mut self, angle: Rotation) -> &mut Self {
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
        let mut mat = self.angle.transformation_matrix();
        if self.reflect_vert {
            mat[0][1] = -mat[0][1];
            mat[1][1] = -mat[1][1];
        }
        Transformation {
            mat,
            b: Point::new(self.x, self.y),
        }
    }
}

/// Finds the inverse of the matrix.
///
/// The determinant factor is unecessary since all transformation matrices have determinant 1 (no
/// scaling).
fn unitary_matinv(a: &[[i8; 2]; 2]) -> [[i8; 2]; 2] {
    [[a[1][1], -a[0][1]], [-a[1][0], a[0][0]]]
}

/// A trait for specifying how an object is changed by a [`Transformation`].
pub trait TransformRef: TranslateRef {
    /// Applies matrix-vector [`Transformation`] `trans`.
    fn transform_ref(&self, trans: Transformation) -> Self;
}

impl TransformRef for () {
    fn transform_ref(&self, _trans: Transformation) -> Self {}
}

impl<T: TransformRef> TransformRef for Vec<T> {
    fn transform_ref(&self, trans: Transformation) -> Self {
        self.iter().map(|elt| elt.transform_ref(trans)).collect()
    }
}

impl<T: TransformRef> TransformRef for Option<T> {
    fn transform_ref(&self, trans: Transformation) -> Self {
        self.as_ref().map(move |elt| elt.transform_ref(trans))
    }
}

/// A trait for specifying how an object is changed by a [`Transformation`].
#[impl_for_tuples(32)]
pub trait TransformMut {
    /// Applies matrix-vector [`Transformation`] `trans`.
    fn transform_mut(&mut self, trans: Transformation);
}

impl<T: TransformMut> TransformMut for Vec<T> {
    fn transform_mut(&mut self, trans: Transformation) {
        for i in self.iter_mut() {
            i.transform_mut(trans);
        }
    }
}

impl<T: TransformMut> TransformMut for Option<T> {
    fn transform_mut(&mut self, trans: Transformation) {
        if let Some(inner) = self.as_mut() {
            inner.transform_mut(trans);
        }
    }
}

/// A trait for specifying how an object is changed by a [`Transformation`].
///
/// Takes in an owned copy of the shape and returns the transformed version.
pub trait Transform: TransformMut + Sized {
    /// Applies matrix-vector [`Transformation`] `trans`.
    ///
    /// Creates a new shape at a location equal to the transformation of the original.
    #[inline]
    fn transform(mut self, trans: Transformation) -> Self {
        self.transform_mut(trans);
        self
    }
}

impl<T: TransformMut + Sized> Transform for T {}

/// A trait for specifying how a shape is translated by a [`Point`].
pub trait TranslateRef: Sized {
    /// Translates the shape by [`Point`], returning a new shape.
    fn translate_ref(&self, p: Point) -> Self;
}

impl TranslateRef for () {
    fn translate_ref(&self, _p: Point) -> Self {}
}

impl<T: TranslateRef> TranslateRef for Vec<T> {
    fn translate_ref(&self, p: Point) -> Self {
        self.iter().map(|elt| elt.translate_ref(p)).collect()
    }
}

impl<T: TranslateRef> TranslateRef for Option<T> {
    fn translate_ref(&self, p: Point) -> Self {
        self.as_ref().map(move |elt| elt.translate_ref(p))
    }
}

/// A trait for specifying how a shape is translated by a [`Point`].
#[impl_for_tuples(32)]
pub trait TranslateMut {
    /// Translates the shape by a [`Point`] through mutation.
    fn translate_mut(&mut self, p: Point);
}

impl<T: TranslateMut> TranslateMut for Vec<T> {
    fn translate_mut(&mut self, p: Point) {
        for i in self.iter_mut() {
            i.translate_mut(p);
        }
    }
}

impl<T: TranslateMut> TranslateMut for Option<T> {
    fn translate_mut(&mut self, p: Point) {
        if let Some(inner) = self.as_mut() {
            inner.translate_mut(p);
        }
    }
}

/// A trait for specifying how a shape is translated by a [`Point`].
///
/// Takes in an owned copy of the shape and returns the translated version.
pub trait Translate: TranslateMut + Sized {
    /// Translates the shape by a [`Point`] through mutation.
    ///
    /// Creates a new shape at a location equal to the translation of the original.
    fn translate(mut self, p: Point) -> Self {
        self.translate_mut(p);
        self
    }
}

impl<T: TranslateMut + Sized> Translate for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{orientation::NamedOrientation, rect::Rect};

    #[test]
    fn matvec_works() {
        let a = [[1, 2], [3, 4]];
        let b = [5, 6];
        assert_eq!(matvec_i8_i64(&a, &b), [17, 39]);
    }

    #[test]
    fn matmul_works() {
        let a = [[1, 2], [3, 4]];
        let b = [[5, 6], [7, 8]];
        assert_eq!(matmul_i8(&a, &b), [[19, 22], [43, 50]]);
    }

    #[test]
    fn unitary_matinv_works() {
        let a = [[1, 1], [3, 4]];
        let inv = unitary_matinv(&a);
        let a_mul_inv = matmul_i8(&a, &inv);
        assert_eq!(a_mul_inv[0][0], 1);
        assert_eq!(a_mul_inv[0][1], 0);
        assert_eq!(a_mul_inv[1][0], 0);
        assert_eq!(a_mul_inv[1][1], 1);
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

        let pt_reflect_vert = pt.transform(Transformation::from_offset_and_orientation(
            Point::zero(),
            NamedOrientation::ReflectVert,
        ));
        assert_eq!(pt_reflect_vert, Point::new(2, -1));

        let pt_reflect_horiz = pt.transform(Transformation::from_offset_and_orientation(
            Point::zero(),
            NamedOrientation::ReflectHoriz,
        ));
        assert_eq!(pt_reflect_horiz, Point::new(-2, 1));

        let pt_r90 = pt.transform(Transformation::from_offset_and_orientation(
            Point::new(23, 11),
            NamedOrientation::R90,
        ));
        assert_eq!(pt_r90, Point::new(22, 13));

        let pt_r180 = pt.transform(Transformation::from_offset_and_orientation(
            Point::new(-50, 10),
            NamedOrientation::R180,
        ));
        assert_eq!(pt_r180, Point::new(-52, 9));

        let pt_r270 = pt.transform(Transformation::from_offset_and_orientation(
            Point::new(80, 90),
            NamedOrientation::R270,
        ));
        assert_eq!(pt_r270, Point::new(81, 88));

        let pt_r90cw = pt.transform(Transformation::from_offset_and_orientation(
            Point::new(5, 13),
            NamedOrientation::R90Cw,
        ));
        assert_eq!(pt_r90cw, Point::new(6, 11));

        let pt_r180cw = pt.transform(Transformation::from_offset_and_orientation(
            Point::zero(),
            NamedOrientation::R180Cw,
        ));
        assert_eq!(pt_r180cw, Point::new(-2, -1));

        let pt_r270cw = pt.transform(Transformation::from_offset_and_orientation(
            Point::new(1, 100),
            NamedOrientation::R270Cw,
        ));
        assert_eq!(pt_r270cw, Point::new(0, 102));

        let pt_flip_yx = pt.transform(Transformation::from_offset_and_orientation(
            Point::new(-65, -101),
            NamedOrientation::FlipYx,
        ));
        assert_eq!(pt_flip_yx, Point::new(-64, -99));

        let pt_flip_minus_yx = pt.transform(Transformation::from_offset_and_orientation(
            Point::new(1, -5),
            NamedOrientation::FlipMinusYx,
        ));
        assert_eq!(pt_flip_minus_yx, Point::new(0, -7));
    }

    #[test]
    fn translate_works_for_tuples() {
        let mut tuple = (
            Rect::from_sides(0, 0, 100, 200),
            Rect::from_sides(50, -50, 150, 0),
        );
        tuple.translate_mut(Point::new(5, 10));
        assert_eq!(
            tuple,
            (
                Rect::from_sides(5, 10, 105, 210),
                Rect::from_sides(55, -40, 155, 10)
            )
        );
    }

    #[test]
    fn translate_works_for_vecs() {
        let mut v = vec![
            Rect::from_sides(0, 0, 100, 200),
            Rect::from_sides(50, -50, 150, 0),
        ];
        v.translate_mut(Point::new(5, 10));
        assert_eq!(
            v,
            vec![
                Rect::from_sides(5, 10, 105, 210),
                Rect::from_sides(55, -40, 155, 10)
            ]
        );
    }

    #[test]
    fn transform_works_for_tuples() {
        let mut tuple = (
            Rect::from_sides(0, 0, 100, 200),
            Rect::from_sides(50, -50, 150, 0),
        );
        tuple.transform_mut(Transformation::from_offset_and_orientation(
            Point::zero(),
            NamedOrientation::R90,
        ));
        assert_eq!(
            tuple,
            (
                Rect::from_sides(-200, 0, 0, 100),
                Rect::from_sides(0, 50, 50, 150)
            )
        );
    }

    #[test]
    fn transform_works_for_vecs() {
        let mut v = vec![
            Rect::from_sides(0, 0, 100, 200),
            Rect::from_sides(50, -50, 150, 0),
        ];
        v.transform_mut(Transformation::from_offset_and_orientation(
            Point::zero(),
            NamedOrientation::R90,
        ));
        assert_eq!(
            v,
            vec![
                Rect::from_sides(-200, 0, 0, 100),
                Rect::from_sides(0, 50, 50, 150)
            ]
        );
    }
}
