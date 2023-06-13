use std::sync::Arc;

use arcstr::ArcStr;

use super::{
    instance::Instance,
    interface::{AnalogInterface, Connectable, Port, SignalMap},
    HasSchematic,
};

pub trait SchematicCell<T>
where
    T: HasSchematic,
{
    fn new(intf: <T::Interface as AnalogInterface<T>>::Uninitialized) -> Self;
    fn add_instance<I, N>(&mut self, name: N, inst: Arc<I::Cell>) -> I::Instance
    where
        I: HasSchematic,
        N: Into<ArcStr>;
    fn instances(&self) -> &Vec<Instance>;
    fn signal_map(&self) -> &SignalMap;
    fn signal_map_mut(&mut self) -> &mut SignalMap;
    fn register_port(&mut self) -> Port {
        self.signal_map_mut().register_port()
    }

    fn interface(&self) -> &T::Interface;

    fn initialize_interface<I>(
        &mut self,
        intf: <I::Interface as AnalogInterface<I>>::Uninitialized,
    ) -> I::Interface
    where
        I: HasSchematic,
    {
        I::Interface::initialize(intf, self.signal_map_mut())
    }

    fn connect(&mut self, a: impl Connectable, b: impl Connectable) {
        self.signal_map_mut().connect(a, b);
    }
}
