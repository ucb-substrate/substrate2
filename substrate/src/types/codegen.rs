//! Helper traits and types used for type codegen.

use std::marker::PhantomData;

use crate::{
    schematic::InstancePath,
    simulation::{data::Save, Analysis, Simulator},
};

use super::{
    layout::{PortGeometry, PortGeometryBundle},
    schematic::{HasNestedView, NestedView, SchematicBundleKind},
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

pub struct DirectView;
pub struct DerivedView;

pub trait ViewSource {
    type Source;
}

pub trait HasViewImpl<V, S> {
    type View;
}

impl<V, S, T: ViewSource<Source = S> + HasViewImpl<V, S>> HasView<V> for T {
    type View = T::View;
}

impl ViewSource for Signal {
    type Source = DirectView;
}

impl<S> HasViewImpl<PortGeometryBundle<S>, DirectView> for Signal {
    type View = PortGeometry<S>;
}

impl<T> ViewSource for Array<T> {
    type Source = DirectView;
}

macro_rules! impl_direction {
    ($dir:ident) => {
        impl<T> ViewSource for $dir<T> {
            type Source = DerivedView;
        }

        impl<V, T: HasView<V>> HasViewImpl<V, DerivedView> for $dir<T> {
            type View = T::View;
        }
    };
}

impl_direction!(InOut);
impl_direction!(Input);
impl_direction!(Output);
impl_direction!(Flipped);

/// Marker struct for nested views.
pub struct Nested<T = InstancePath>(PhantomData<T>);

impl<T, D: HasNestedView<T>> HasViewImpl<Nested<T>, DirectView> for D {
    type View = <D as HasNestedView<T>>::NestedView;
}

pub struct NodeBundle;
pub struct TerminalBundle;
pub struct NestedNodeBundle;
pub struct NestedTerminalBundle;

impl<T: SchematicBundleKind> HasViewImpl<NodeBundle, DirectView> for T {
    type View = <T as SchematicBundleKind>::NodeBundle;
}

impl<T: SchematicBundleKind> HasViewImpl<TerminalBundle, DirectView> for T {
    type View = <T as SchematicBundleKind>::TerminalBundle;
}

impl<T: SchematicBundleKind> HasViewImpl<NestedNodeBundle, DirectView> for T {
    type View = NestedView<<T as SchematicBundleKind>::NodeBundle>;
}

impl<T: SchematicBundleKind> HasViewImpl<NestedTerminalBundle, DirectView> for T {
    type View = NestedView<<T as SchematicBundleKind>::TerminalBundle>;
}

pub struct SaveKey<S, A>(PhantomData<(S, A)>);
pub struct Saved<S, A>(PhantomData<(S, A)>);

impl<S: Simulator, A: Analysis, T: Save<S, A>> HasViewImpl<SaveKey<S, A>, DirectView> for T {
    type View = crate::simulation::data::SaveKey<T, S, A>;
}
