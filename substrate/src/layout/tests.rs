#![allow(dead_code)]
use substrate::geometry::transform::{TransformMut, TranslateMut};
use substrate::layout::Instance;

use super::Layout;

#[derive(Default, TranslateMut, TransformMut)]
pub struct LayoutInstances<T: Layout> {
    pub instances: Vec<Instance<T>>,
}

#[derive(TransformMut, TranslateMut)]
pub enum EnumInstances<T: Layout> {
    One { one: Instance<T> },
    Two(Instance<T>, Instance<T>),
}

#[derive(TransformMut, TranslateMut)]
pub struct TwoInstances<T: Layout>(pub Instance<T>, pub Instance<T>);
