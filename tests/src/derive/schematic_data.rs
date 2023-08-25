use substrate::pdk::Pdk;
use substrate::schematic::schema::Schema;
use substrate::schematic::{ExportsSchematicData, Instance, SchematicData};

#[derive(Default, SchematicData)]
pub struct SchematicInstances<PDK: Pdk, S: Schema, T: ExportsSchematicData<PDK, S>> {
    pub instances: Vec<Instance<PDK, S, T>>,
}

#[derive(SchematicData)]
pub enum EnumInstances<PDK: Pdk, S: Schema, T: ExportsSchematicData<PDK, S>> {
    One { one: Instance<PDK, S, T> },
    Two(Instance<PDK, S, T>, Instance<PDK, S, T>),
}

#[derive(SchematicData)]
pub struct TwoInstances<PDK: Pdk, S: Schema, T: ExportsSchematicData<PDK, S>>(
    pub Instance<PDK, S, T>,
    pub Instance<PDK, S, T>,
);
