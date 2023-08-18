use substrate::layout::{ExportsLayoutData, Instance, LayoutData};

#[derive(Default, LayoutData)]
pub struct LayoutInstances<T: ExportsLayoutData> {
    pub instances: Vec<Instance<T>>,
}

#[derive(LayoutData)]
pub enum EnumInstances<T: ExportsLayoutData> {
    One { one: Instance<T> },
    Two(Instance<T>, Instance<T>),
}

#[derive(LayoutData)]
pub struct TwoInstances<T: ExportsLayoutData>(pub Instance<T>, pub Instance<T>);
