//! Helper traits and types used for type codegen.

use std::marker::PhantomData;

use crate::{
    schematic::{HasNestedView, InstancePath, NestedView},
    simulation::{data::Save, Analysis, Simulator},
};

use super::{
    layout::{PortGeometry, PortGeometryBundle},
    schematic::SchematicBundleKind,
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
pub struct FromSource<T>(PhantomData<T>);

pub trait Source {}

impl Source for FromSelf {}
impl<T> Source for FromSource<T> {}

pub trait ViewSource {
    type Source: Source;
}

pub trait HasViewImpl<V, S = FromSelf> {
    type View;
}

impl<S: HasView<V>, V, T: ViewSource<Source = FromSource<S>>> HasViewImpl<V, FromSource<S>> for T {
    type View = S::View;
}

impl<V, S, T: ViewSource<Source = S> + HasViewImpl<V, S>> HasView<V> for T {
    type View = T::View;
}

impl ViewSource for Signal {
    type Source = FromSelf;
}

impl<S> HasViewImpl<PortGeometryBundle<S>, FromSelf> for Signal {
    type View = PortGeometry<S>;
}

impl<T> ViewSource for Array<T> {
    type Source = FromSelf;
}

macro_rules! impl_direction {
    ($dir:ident) => {
        impl<T> ViewSource for $dir<T> {
            type Source = FromSource<T>;
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

pub struct SaveKey<S, A>(PhantomData<(S, A)>);
pub struct Saved<S, A>(PhantomData<(S, A)>);

impl<S: Simulator, A: Analysis, T: Save<S, A>> HasViewImpl<SaveKey<S, A>> for T {
    type View = crate::simulation::data::SaveKey<T, S, A>;
}
