use substrate::schematic::{ExportsNestedData, Instance, NestedData};

#[derive(Default, NestedData)]
pub struct SchematicInstances<T: ExportsNestedData> {
    pub instances: Vec<Instance<T>>,
}

#[derive(NestedData)]
pub enum EnumInstances<T: ExportsNestedData> {
    One { one: Instance<T> },
    Two(Instance<T>, Instance<T>),
}

#[derive(NestedData)]
pub struct TwoInstances<T: ExportsNestedData>(pub Instance<T>, pub Instance<T>);
