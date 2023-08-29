use substrate::pdk::Pdk;
use substrate::schematic::schema::Schema;
use substrate::schematic::{ExportsNestedNodes, Instance, InstanceData, SchematicData};

#[derive(Default, SchematicData)]
pub struct SchematicInstances<T: ExportsNestedNodes> {
    pub instances: Vec<InstanceData<T>>,
}

#[derive(SchematicData)]
pub enum EnumInstances<T: ExportsNestedNodes> {
    One { one: InstanceData<T> },
    Two(InstanceData<T>, InstanceData<T>),
}

#[derive(SchematicData)]
pub struct TwoInstances<T: ExportsNestedNodes>(pub InstanceData<T>, pub InstanceData<T>);
