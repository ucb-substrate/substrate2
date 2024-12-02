use std::hash::Hash;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Id<T>(u64, std::marker::PhantomData<T>);

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T> Eq for Id<T> {}

impl<T> Id<T> {
    pub(crate) fn new() -> Self {
        Self(0, std::marker::PhantomData)
    }

    pub(crate) fn alloc(&mut self) -> Self {
        *self = Self(self.0 + 1, std::marker::PhantomData);
        *self
    }
}

impl<T> Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
