use substrate::pdk::Pdk;
use substrate::schematic::schema::Schema;
use substrate::schematic::{ExportsNestedData, Instance, SchematicData};

#[derive(Default, SchematicData)]
pub struct SchematicInstances<T: ExportsNestedData> {
    pub instances: Vec<Instance<T>>,
}

#[derive(SchematicData)]
pub enum EnumInstances<T: ExportsNestedData> {
    One { one: Instance<T> },
    Two(Instance<T>, Instance<T>),
}

#[derive(SchematicData)]
pub struct TwoInstances<T: ExportsNestedData>(pub Instance<T>, pub Instance<T>);
