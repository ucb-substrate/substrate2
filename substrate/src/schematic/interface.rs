use std::fmt::Debug;

use slotmap::{new_key_type, SlotMap};

use super::HasSchematic;

new_key_type! { pub struct SignalKey; }

pub struct Net(usize);

#[derive(Default, Debug, Clone, Copy)]
pub struct Port {
    key: SignalKey,
}

impl Port {
    pub(crate) fn new(key: SignalKey) -> Self {
        Self { key }
    }
}

pub trait AnalogInterface<T>: Clone + Debug
where
    T: HasSchematic,
{
    type Uninitialized;

    fn initialize(intf: Self::Uninitialized, map: &mut SignalMap) -> Self;
    fn uninitialized(self) -> Self::Uninitialized;
}

pub trait Connectable {
    fn key(&self) -> SignalKey;
}

impl Connectable for Port {
    fn key(&self) -> SignalKey {
        self.key
    }
}

impl Connectable for SignalKey {
    fn key(&self) -> SignalKey {
        *self
    }
}

#[derive(Debug, Clone)]
struct SignalData {
    group: SignalKey,
    weight: usize,
}

#[derive(Default, Debug, Clone)]
pub struct SignalMap {
    map: SlotMap<SignalKey, SignalData>,
}

impl SignalMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn register_signal(&mut self) -> SignalKey {
        self.map.insert_with_key(|k| SignalData {
            group: k,
            weight: 1,
        })
    }

    pub fn register_port(&mut self) -> Port {
        Port::new(self.register_signal())
    }

    fn find(&self, mut key: SignalKey) -> SignalKey {
        while self.map[key].group != key {
            key = self.map[key].group
        }
        key
    }

    fn union(&mut self, a: SignalKey, b: SignalKey) {
        let a_group = self.find(a);
        let b_group = self.find(b);

        if a_group == b_group {
            return;
        }

        let a_weight = self.map[a_group].weight;
        let b_weight = self.map[b_group].weight;

        if a_weight > b_weight {
            self.map[b_group].group = a_group;
            self.map[a_group].weight = a_weight + b_weight;
        } else {
            self.map[a_group].group = b_group;
            self.map[b_group].weight = a_weight + b_weight;
        }
    }

    pub fn connect(&mut self, a: impl Connectable, b: impl Connectable) {
        self.union(a.key(), b.key());
    }

    pub fn connected(&mut self, a: impl Connectable, b: impl Connectable) -> bool {
        self.find(a.key()) == self.find(b.key())
    }
}
