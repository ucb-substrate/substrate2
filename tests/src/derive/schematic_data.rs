use substrate::schematic::{HasSchematicData, Instance};
use substrate::SchematicData;

#[derive(Default, SchematicData)]
pub struct SchematicInstances<T: HasSchematicData> {
    #[substrate(nested)]
    pub instances: Vec<Instance<T>>,
    pub field: i64,
}

#[derive(SchematicData)]
pub enum EnumInstances<T: HasSchematicData> {
    One {
        #[substrate(nested)]
        one: Instance<T>,
        field: i64,
    },
    Two(
        #[substrate(nested)] Instance<T>,
        #[substrate(nested)] Instance<T>,
        i64,
    ),
}

#[derive(SchematicData)]
pub struct TwoInstances<T: HasSchematicData>(
    #[substrate(nested)] pub Instance<T>,
    #[substrate(nested)] pub Instance<T>,
    pub i64,
);
