use geometry::transform::TransformMut;
use substrate::layout::{ExportsLayoutData, Instance, LayoutData};

#[derive(Default, TransformMut)]
pub struct LayoutInstances<T: ExportsLayoutData> {
    pub instances: Vec<Instance<T>>,
}

#[derive(TransformMut)]
pub enum EnumInstances<T: ExportsLayoutData> {
    One { one: Instance<T> },
    Two(Instance<T>, Instance<T>),
}

#[derive(TransformMut)]
pub struct TwoInstances<T: ExportsLayoutData>(pub Instance<T>, pub Instance<T>);
