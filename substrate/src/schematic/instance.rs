use std::{collections::HashMap, sync::Arc};

use arcstr::ArcStr;

use super::{interface::Port, HasSchematic};

#[derive(Debug, Clone)]
pub struct Instance {
    pub name: ArcStr,
    pub instances: Vec<Instance>,
    pub ports: HashMap<ArcStr, Port>,
}

pub trait SchematicInstance<T>: Into<Instance> + Clone
where
    T: HasSchematic,
{
    fn new(
        name: impl Into<ArcStr>,
        intf: <T as HasSchematic>::Interface,
        cell: Arc<<T as HasSchematic>::Cell>,
    ) -> Self;
    fn name(&self) -> ArcStr;
    fn intf(&self) -> &<T as HasSchematic>::Interface;
}
