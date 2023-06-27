use std::ops::Deref;

use geometry::{
    prelude::Point,
    rect::Rect,
    transform::{Transform, Transformation},
};

use crate::transformed::HasLayout;

pub trait HasTransformedView {
    type TransformedView;

    fn transformed_view(&self, trans: Transformation) -> Self::TransformedView;
}

pub type Transformed<T> = <T as HasTransformedView>::TransformedView;

pub struct TransformedPrimitive<'a, T> {
    inner: &'a T,
    trans: &'a Transformation,
}

impl HasTransformedView for Rect {
    type TransformedView = Rect;

    fn transformed_view(&self, trans: Transformation) -> Self::TransformedView {
        self.transform(trans)
    }
}

pub struct TransformedVec<'a, T> {
    inner: &'a Vec<T>,
    trans: Transformation,
}

impl<'a, T> HasTransformedView for &'a Vec<T> {
    type TransformedView = TransformedVec<'a, T>;

    fn transformed_view(&self, trans: Transformation) -> Self::TransformedView {
        TransformedVec { inner: self, trans }
    }
}

impl<'a, T: HasTransformedView> TransformedVec<'a, T> {
    fn get(&self, idx: usize) -> Option<Self> {
        None
    }
}
