//! Helper traits and types used for type codegen.

use std::marker::PhantomData;

use crate::{
    schematic::{HasNestedView, Instance, InstancePath, NestedInstance, NestedView, Schematic},
    simulation::{data::Save, Analysis, Simulator},
};

use super::{
    layout::{LayoutBundle, PortGeometry},
    schematic::{
        HasNodeBundle, HasTerminalBundle, NestedNode, NestedTerminal, Node, SchematicBundleKind,
        Terminal,
    },
    Array, ArrayBundle, Flipped, HasBundleKind, InOut, Input, Output, Signal,
};

/// A type with an associated `V` view.
///
/// `V` is generally a zero-size marker struct.
pub trait HasView<V> {
    /// The associated view.
    type View;
}

/// The `V` view of `T`.
pub type View<T, V> = <T as HasView<V>>::View;

/// Marker struct for nested views.
pub struct Nested<T = InstancePath>(PhantomData<T>);

impl<T, D> HasView<Nested<T>> for D
where
    D: HasNestedView<T>,
{
    type View = <D as HasNestedView<T>>::NestedView;
}

pub struct NodeBundle;
pub struct TerminalBundle;
pub struct NestedNodeBundle;
pub struct NestedTerminalBundle;

impl<T: HasNodeBundle> HasView<NodeBundle> for T {
    type View = <T as HasNodeBundle>::NodeBundle;
}

impl<T: HasTerminalBundle> HasView<TerminalBundle> for T {
    type View = <T as HasTerminalBundle>::TerminalBundle;
}

impl<T: HasNodeBundle> HasView<NestedNodeBundle> for T {
    type View = NestedView<<T as HasNodeBundle>::NodeBundle>;
}

impl<T: HasTerminalBundle> HasView<NestedTerminalBundle> for T {
    type View = NestedView<<T as HasTerminalBundle>::TerminalBundle>;
}

pub trait HasSchematicBundleKindViews:
    HasBundleKind<BundleKind: SchematicBundleKind>
    + HasView<NodeBundle, View = super::schematic::NodeBundle<Self>>
    + HasView<TerminalBundle, View = super::schematic::TerminalBundle<Self>>
    + HasView<NestedNodeBundle, View = NestedView<super::schematic::NodeBundle<Self>>>
    + HasView<NestedTerminalBundle, View = NestedView<super::schematic::TerminalBundle<Self>>>
    + HasView<NestedNodeBundle, View: Send + Sync>
    + HasView<NestedTerminalBundle, View: Send + Sync>
{
}

impl<
        T: HasBundleKind<BundleKind: SchematicBundleKind>
            + HasView<NodeBundle, View = super::schematic::NodeBundle<T>>
            + HasView<TerminalBundle, View = super::schematic::TerminalBundle<T>>
            + HasView<NestedNodeBundle, View = NestedView<super::schematic::NodeBundle<Self>>>
            + HasView<NestedTerminalBundle, View = NestedView<super::schematic::TerminalBundle<Self>>>,
    > HasSchematicBundleKindViews for T
{
}

pub struct NestedSaveKey<T, S, A>(PhantomData<(T, S, A)>);
pub struct NestedSaved<T, S, A>(PhantomData<(T, S, A)>);

impl<V, S: Simulator, A: Analysis, T: HasNestedView<V, NestedView: Save<S, A>>>
    HasView<NestedSaveKey<V, S, A>> for T
{
    type View = crate::simulation::data::SaveKey<NestedView<T, V>, S, A>;
}

impl<V, S: Simulator, A: Analysis, T: HasNestedView<V, NestedView: Save<S, A>>>
    HasView<NestedSaved<V, S, A>> for T
{
    type View = crate::simulation::data::Saved<NestedView<T, V>, S, A>;
}

pub struct SaveKeyView<V, S, A>(PhantomData<(V, S, A)>);
pub struct SavedView<V, S, A>(PhantomData<(V, S, A)>);

// impl<V, S, A, T> HasViewImpl<SaveKeyView<V, S, A>> for T
// where
//     S: Simulator,
//     A: Analysis,
//     T: HasView<V>,
//     <T as HasView<V>>::View: Save<S, A>,
// {
//     type View = crate::simulation::data::SaveKey<<T as HasView<V>>::View, S, A>;
// }

pub trait HasDefaultLayoutBundle: super::BundleKind {
    type Bundle<S: crate::layout::schema::Schema>: LayoutBundle<S>;
}
/// A port geometry bundle view.
pub struct PortGeometryBundle<S>(PhantomData<S>);

impl<S: crate::layout::schema::Schema, T: HasDefaultLayoutBundle> HasView<PortGeometryBundle<S>>
    for Array<T>
{
    type View = ArrayBundle<T::Bundle<S>>;
}

impl<T: HasDefaultLayoutBundle> HasDefaultLayoutBundle for Array<T> {
    type Bundle<S: crate::layout::schema::Schema> = ArrayBundle<T::Bundle<S>>;
}

impl HasDefaultLayoutBundle for Signal {
    type Bundle<S: crate::layout::schema::Schema> = PortGeometry<S::Layer>;
}
