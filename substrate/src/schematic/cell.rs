use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use arcstr::ArcStr;

use super::{
    instance::{Instance, SchematicInstance},
    interface::{AnalogInterface, Connectable, Port, SignalMap},
    HasSchematic,
};

#[derive(Debug)]
pub struct SchematicCell<T>
where
    T: HasSchematic,
{
    intf: T::Interface,
    instances: InstanceMap,
    signal_map: SignalMap,
}

impl<T> SchematicCell<T>
where
    T: HasSchematic,
{
    pub fn new(intf: <T::Interface as AnalogInterface<T>>::Uninitialized) -> Self {
        let mut map = SignalMap::new();
        SchematicCell {
            intf: T::Interface::initialize(intf, &mut map),
            instances: InstanceMap::new(),
            signal_map: map,
        }
    }

    pub fn add_instance<I, N>(
        &mut self,
        name: N,
        cell: Arc<SchematicCell<I>>,
    ) -> SchematicInstance<I>
    where
        I: 'static + HasSchematic,
        N: Into<ArcStr>,
    {
        let intf = self.initialize_interface::<I>(cell.intf().clone().uninitialized());
        let inst = SchematicInstance::new(name, intf, cell);
        self.instances.add_instance(inst.clone());
        inst
    }

    pub fn register_port(&mut self) -> Port {
        self.signal_map.register_port()
    }

    pub fn intf(&self) -> &T::Interface {
        &self.intf
    }

    pub(crate) fn initialize_interface<I>(
        &mut self,
        intf: <I::Interface as AnalogInterface<I>>::Uninitialized,
    ) -> I::Interface
    where
        I: HasSchematic,
    {
        I::Interface::initialize(intf, &mut self.signal_map)
    }

    pub fn connect(&mut self, a: impl Connectable, b: impl Connectable) {
        self.signal_map.connect(a, b);
    }

    pub(crate) fn instances(&self) -> impl Iterator<Item = &Instance> {
        self.instances.str_map.values()
    }

    pub fn instance_map(&self) -> &InstanceMap {
        &self.instances
    }

    pub fn signal_map(&self) -> &SignalMap {
        &self.signal_map
    }
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
