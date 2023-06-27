use std::{any::Any, ops::Deref};

use geometry::{
    prelude::Point,
    rect::Rect,
    transform::{Transform, Transformation},
};

use crate::transformed::HasLayout;

pub trait HasTransformedView {
    type TransformedView<'a>
    where
        Self: 'a;

    fn transformed_view<'a>(&'a self, trans: Transformation) -> Self::TransformedView<'a>;
}

pub type Transformed<'a, T> = <T as HasTransformedView>::TransformedView<'a>;

pub struct TransformedPrimitive<'a, T> {
    inner: &'a T,
    trans: &'a Transformation,
}

impl HasTransformedView for Rect {
    type TransformedView<'a> = Rect;

    fn transformed_view<'a>(&'a self, trans: Transformation) -> Self::TransformedView<'a> {
        self.transform(trans)
    }
}

pub struct TransformedVec<'a, T> {
    inner: &'a Vec<T>,
    trans: Transformation,
}

impl<T: Any> HasTransformedView for Vec<T> {
    type TransformedView<'a> = TransformedVec<'a, T>;

    fn transformed_view<'a>(&'a self, trans: Transformation) -> Self::TransformedView<'a> {
        TransformedVec { inner: self, trans }
    }
}

impl<'a, T: HasTransformedView> TransformedVec<'a, T> {
    pub fn get(&self, idx: usize) -> Option<T::TransformedView<'a>> {
        self.inner.get(idx).map(|i| i.transformed_view(self.trans))
    }
}
