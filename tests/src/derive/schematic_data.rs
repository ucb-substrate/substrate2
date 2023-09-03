use substrate::pdk::Pdk;
use substrate::schematic::schema::Schema;
use substrate::schematic::{ExportsNestedData, Instance, InstanceData, SchematicData};

#[derive(Default, SchematicData)]
pub struct SchematicInstances<T: ExportsNestedData> {
    pub instances: Vec<InstanceData<T>>,
}

#[derive(SchematicData)]
pub enum EnumInstances<T: ExportsNestedData> {
    One { one: InstanceData<T> },
    Two(InstanceData<T>, InstanceData<T>),
}

#[derive(SchematicData)]
pub struct TwoInstances<T: ExportsNestedData>(pub InstanceData<T>, pub InstanceData<T>);
