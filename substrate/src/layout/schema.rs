use std::{any::Any, fmt::Debug};

pub trait Schema {
    type Layer: Debug + Send + Sync + Any + Clone + Eq;
}
