//! Helper traits and types used for type codegen.

use std::marker::PhantomData;

use crate::{
    schematic::{HasNestedView, Instance, InstancePath, NestedInstance, NestedView, Schematic},
    simulation::{data::Save, Analysis, Simulator},
};

use super::{
    layout::{LayoutBundle, PortGeometry},
    schematic::{NestedNode, NestedTerminal, Node, SchematicBundleKind, Terminal},
    Array, ArrayBundle, Flipped, HasBundleKind, InOut, Input, Output, Signal,
};

/// A type with an associated `V` view.
///
/// `V` is generally a zero-size marker struct.
pub trait HasView<V>: ViewSource {
    /// The associated view.
    type View;
}

/// The `V` view of `D`.
pub type View<D, V> = <D as HasView<V>>::View;

pub struct FromSelf;
pub struct FromOther;

pub trait ViewSource {
    type Kind;
    type Source;
}

pub trait HasViewImpl<V, S = FromSelf>: ViewSource {
    type View;
}

impl<S: HasView<V>, V, T: ViewSource<Kind = FromOther, Source = S>> HasViewImpl<V, FromOther>
    for T
{
    type View = S::View;
}

impl<V, K, T: ViewSource<Kind = K> + HasViewImpl<V, K>> HasView<V> for T {
    type View = T::View;
}

impl ViewSource for () {
    type Kind = FromSelf;
    type Source = Self;
}

impl<T> ViewSource for Option<T> {
    type Kind = FromSelf;
    type Source = Self;
}

impl ViewSource for Signal {
    type Kind = FromSelf;
    type Source = Self;
}

impl<L> HasViewImpl<PortGeometryBundle<L>> for Signal {
    type View = PortGeometry<L>;
}

impl ViewSource for Node {
    type Kind = FromSelf;
    type Source = Self;
}

impl ViewSource for Terminal {
    type Kind = FromSelf;
    type Source = Self;
}

impl ViewSource for NestedNode {
    type Kind = FromSelf;
    type Source = Self;
}

impl ViewSource for NestedTerminal {
    type Kind = FromSelf;
    type Source = Self;
}

impl<T: ViewSource> ViewSource for Array<T> {
    type Kind = T::Kind;
    type Source = Array<T::Source>;
}

impl<T: Schematic> ViewSource for Instance<T> {
    type Kind = FromSelf;
    type Source = Self;
}

impl<T: Schematic> ViewSource for NestedInstance<T> {
    type Kind = FromSelf;
    type Source = Self;
}

impl<T: ViewSource> ViewSource for Vec<T> {
    type Kind = T::Kind;
    type Source = Vec<T::Source>;
}

macro_rules! impl_direction {
    ($dir:ident) => {
        impl<T: ViewSource> ViewSource for $dir<T> {
            type Kind = FromOther;
            type Source = T::Source;
        }
    };
}

impl_direction!(InOut);
impl_direction!(Input);
impl_direction!(Output);
impl_direction!(Flipped);

/// Marker struct for nested views.
pub struct Nested<T = InstancePath>(PhantomData<T>);

impl<T, D: ViewSource + HasNestedView<T>> HasViewImpl<Nested<T>> for D {
    type View = <D as HasNestedView<T>>::NestedView;
}

pub struct NodeBundle;
pub struct TerminalBundle;
pub struct NestedNodeBundle;
pub struct NestedTerminalBundle;

impl<T: ViewSource + SchematicBundleKind> HasViewImpl<NodeBundle> for T {
    type View = <T as SchematicBundleKind>::NodeBundle;
}

impl<T: ViewSource + SchematicBundleKind> HasViewImpl<TerminalBundle> for T {
    type View = <T as SchematicBundleKind>::TerminalBundle;
}

impl<T: ViewSource + SchematicBundleKind> HasViewImpl<NestedNodeBundle> for T {
    type View = NestedView<<T as SchematicBundleKind>::NodeBundle>;
}

impl<T: ViewSource + SchematicBundleKind> HasViewImpl<NestedTerminalBundle> for T {
    type View = NestedView<<T as SchematicBundleKind>::TerminalBundle>;
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

impl<V, S: Simulator, A: Analysis, T: ViewSource + HasNestedView<V, NestedView: Save<S, A>>>
    HasViewImpl<NestedSaveKey<V, S, A>> for T
{
    type View = crate::simulation::data::SaveKey<NestedView<T, V>, S, A>;
}

impl<V, S: Simulator, A: Analysis, T: ViewSource + HasNestedView<V, NestedView: Save<S, A>>>
    HasViewImpl<NestedSaved<V, S, A>> for T
{
    type View = crate::simulation::data::Saved<NestedView<T, V>, S, A>;
}

pub trait HasDefaultLayoutBundle: super::BundleKind {
    type Bundle<S: crate::layout::schema::Schema>: LayoutBundle<S>;
}
/// A port geometry bundle view.
pub struct PortGeometryBundle<S>(PhantomData<S>);

impl<S: crate::layout::schema::Schema, T: ViewSource + HasDefaultLayoutBundle>
    HasViewImpl<PortGeometryBundle<S>> for Array<T>
{
    type View = ArrayBundle<T::Bundle<S>>;
}

impl<T: HasDefaultLayoutBundle> HasDefaultLayoutBundle for Array<T> {
    type Bundle<S: crate::layout::schema::Schema> = ArrayBundle<T::Bundle<S>>;
}

impl HasDefaultLayoutBundle for Signal {
    type Bundle<S: crate::layout::schema::Schema> = PortGeometry<S::Layer>;
}
