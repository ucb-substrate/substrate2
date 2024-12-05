use std::any::Any;

pub trait Schema {
    type Layer: Send + Sync + Any + Clone + PartialEq;
    type Data;
}
