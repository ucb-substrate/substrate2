//! Helper traits and types used for type codegen.

use std::marker::PhantomData;

use crate::{
    schematic::{HasNestedView, Instance, InstancePath, NestedInstance, NestedView, Schematic},
    simulation::{data::Save, Analysis, Simulator},
};

use super::{
    layout::{PortGeometry, PortGeometryBundle},
    schematic::{NestedNode, NestedTerminal, Node, SchematicBundleKind, Terminal},
    Array, Flipped, InOut, Input, Output, Signal,
};

/// A type with an associated `V` view.
///
/// `V` is generally a zero-size marker struct.
pub trait HasView<V> {
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

pub trait HasViewImpl<V, S = FromSelf> {
    type View;
}

pub struct Custom<T>(PhantomData<T>);

pub trait HasCustomView<V>: ViewSource<Kind = FromSelf> {
    type View;
}

impl<V, T: HasCustomView<V>> HasViewImpl<Custom<V>> for T {
    type View = T::View;
}

impl<S: HasView<V>, V, T: ViewSource<Kind = FromOther, Source = S>> HasViewImpl<V, FromOther>
    for T
{
    type View = S::View;
}

impl<V, K, T: ViewSource<Kind = K> + HasViewImpl<V, K>> HasView<V> for T {
    type View = T::View;
}

impl ViewSource for Signal {
    type Kind = FromSelf;
    type Source = Self;
}

impl<S> HasViewImpl<PortGeometryBundle<S>> for Signal {
    type View = PortGeometry<S>;
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

impl<T, D: HasNestedView<T>> HasViewImpl<Nested<T>> for D {
    type View = <D as HasNestedView<T>>::NestedView;
}

pub struct NodeBundle;
pub struct TerminalBundle;
pub struct NestedNodeBundle;
pub struct NestedTerminalBundle;

impl<T: SchematicBundleKind> HasViewImpl<NodeBundle> for T {
    type View = <T as SchematicBundleKind>::NodeBundle;
}

impl<T: SchematicBundleKind> HasViewImpl<TerminalBundle> for T {
    type View = <T as SchematicBundleKind>::TerminalBundle;
}

impl<T: SchematicBundleKind> HasViewImpl<NestedNodeBundle> for T {
    type View = NestedView<<T as SchematicBundleKind>::NodeBundle>;
}

impl<T: SchematicBundleKind> HasViewImpl<NestedTerminalBundle> for T {
    type View = NestedView<<T as SchematicBundleKind>::TerminalBundle>;
}

pub struct NestedSaveKey<T, S, A>(PhantomData<(T, S, A)>);
pub struct NestedSaved<T, S, A>(PhantomData<(T, S, A)>);

impl<V, S: Simulator, A: Analysis, T: HasNestedView<V, NestedView: Save<S, A>>>
    HasViewImpl<NestedSaveKey<V, S, A>> for T
{
    type View = crate::simulation::data::SaveKey<NestedView<T, V>, S, A>;
}

impl<V, S: Simulator, A: Analysis, T: HasNestedView<V, NestedView: Save<S, A>>>
    HasViewImpl<NestedSaved<V, S, A>> for T
{
    type View = crate::simulation::data::Saved<NestedView<T, V>, S, A>;
}
