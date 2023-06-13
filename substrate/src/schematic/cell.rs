use super::{
    instance::Instance,
    interface::{AnalogInterface, Connectable, Port, SignalMap},
    HasSchematic,
};

pub trait SchematicCell<T>
where
    T: HasSchematic,
{
    fn new(intf: <T as HasSchematic>::Interface) -> Self;
    fn add_instance(&mut self, inst: impl Into<Instance>);
    fn instances(&self) -> &Vec<Instance>;
    fn signal_map(&self) -> &SignalMap;
    fn signal_map_mut(&mut self) -> &mut SignalMap;
    fn register_port(&mut self) -> Port {
        self.signal_map_mut().register_port()
    }

    fn interface(&self) -> &T::Interface;

    fn register_interface<I>(&mut self, intf: &mut I) -> I
    where
        I: AnalogInterface<T>,
    {
        intf.register(self.signal_map_mut())
    }

    fn connect(&mut self, a: impl Connectable, b: impl Connectable) {
        self.signal_map_mut().connect(a, b);
    }
}
