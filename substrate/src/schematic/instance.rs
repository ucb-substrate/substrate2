use std::{collections::HashMap, sync::Arc};

use arcstr::ArcStr;

use super::{
    cell::SchematicCell,
    interface::{AnalogInterface, Port},
    HasSchematic,
};

#[derive(Debug, Clone)]
pub struct Instance {
    pub name: ArcStr,
    pub instances: Vec<Instance>,
    pub ports: HashMap<ArcStr, Port>,
}

#[derive(Debug)]
pub struct SchematicInstance<T>
where
    T: HasSchematic,
{
    name: ArcStr,
    intf: T::Interface,
    cell: Arc<SchematicCell<T>>,
}

impl<T> Clone for SchematicInstance<T>
where
    T: HasSchematic,
{
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            intf: self.intf.clone(),
            cell: self.cell.clone(),
        }
    }
}

impl<T> SchematicInstance<T>
where
    T: HasSchematic,
{
    pub(crate) fn new(
        name: impl Into<ArcStr>,
        intf: <T as HasSchematic>::Interface,
        cell: Arc<SchematicCell<T>>,
    ) -> Self {
        Self {
            name: name.into(),
            intf,
            cell,
        }
    }

    pub fn name(&self) -> ArcStr {
        self.name.clone()
    }

    pub fn intf(&self) -> &<T as HasSchematic>::Interface {
        &self.intf
    }
}

impl<T> From<SchematicInstance<T>> for Instance
where
    T: HasSchematic,
{
    fn from(value: SchematicInstance<T>) -> Self {
        Self {
            name: value.name,
            instances: value.cell.instances().cloned().collect(),
            ports: value.intf.ports(),
        }
    }
}
