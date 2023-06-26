use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use arcstr::ArcStr;

use crate::block::Block;

use super::{
    instance::{Instance, SchematicInstance},
    interface::{AnalogInterface, Connectable, Port, SignalMap},
    HasSchematic,
};

#[derive(Debug)]
pub struct SchematicCell<T>
where
    T: HasSchematic + Block,
{
    intf: T::Io,
    instances: InstanceMap,
    signal_map: SignalMap,
}

#[derive(Default, Debug)]
pub struct InstanceMap {
    type_map: HashMap<TypeId, Box<dyn Any>>,
    str_map: HashMap<ArcStr, Instance>,
}

impl InstanceMap {
    fn new() -> Self {
        Self::default()
    }

    fn add_instance<T>(&mut self, inst: SchematicInstance<T>)
    where
        T: 'static + HasSchematic,
    {
        if let Some(v) = self
            .type_map
            .entry(TypeId::of::<T>())
            .or_insert(Box::<HashMap<ArcStr, SchematicInstance<T>>>::default())
            .downcast_mut::<HashMap<ArcStr, SchematicInstance<T>>>()
        {
            v.insert(inst.name(), inst.clone());
        }
        self.str_map.insert(inst.name(), inst.into());
    }

    pub fn get_instance<T>(&self, name: &str) -> Option<&SchematicInstance<T>>
    where
        T: 'static + HasSchematic,
    {
        self.type_map
            .get(&TypeId::of::<T>())?
            .downcast_ref::<HashMap<ArcStr, SchematicInstance<T>>>()?
            .get(name)
    }
}
