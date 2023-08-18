use substrate::schematic::{ExportsSchematicData, Instance, SchematicData};

#[derive(Default, SchematicData)]
pub struct SchematicInstances<T: ExportsSchematicData> {
    pub instances: Vec<Instance<T>>,
}

#[derive(SchematicData)]
pub enum EnumInstances<T: ExportsSchematicData> {
    One { one: Instance<T> },
    Two(Instance<T>, Instance<T>),
}

#[derive(SchematicData)]
pub struct TwoInstances<T: ExportsSchematicData>(pub Instance<T>, pub Instance<T>);
