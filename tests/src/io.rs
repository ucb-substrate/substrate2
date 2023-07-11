//! Tests for ensuring that derive(Io) works.

use substrate::io::{Input, LayoutType, SchematicType, Undirected};
use substrate::Io;

#[derive(Debug, Clone, Io)]
pub struct GenericIo<T>
where
    T: Clone + Undirected + SchematicType + LayoutType + 'static,
    <T as SchematicType>::Data: Undirected,
    <T as LayoutType>::Data: Undirected,
    <T as LayoutType>::Builder: Undirected,
{
    pub signal: Input<T>,
}
