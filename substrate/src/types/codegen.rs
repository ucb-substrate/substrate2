//! Helper traits and types used for type codegen.

use std::marker::PhantomData;

use crate::{
    schematic::{ContextView, HasContextView, HasNestedView, NestedView},
    simulation::{Analysis, Simulator, data::Save},
};

use super::{
    Array, ArrayBundle, HasBundleKind, Signal,
    layout::{LayoutBundle, PortGeometry},
    schematic::{HasNodeBundle, HasTerminalBundle, SchematicBundleKind},
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

/// Marker struct for context views.
pub struct Context<T>(PhantomData<T>);

/// Marker struct for nested views.
pub struct Nested;

impl<D> HasView<Nested> for D
where
    D: HasNestedView,
{
    type View = <D as HasNestedView>::NestedView;
}

impl<T, D> HasView<Context<T>> for D
where
    D: HasContextView<T>,
{
    type View = ContextView<D, T>;
}

pub trait HasNestedContextView<T>:
    HasNestedView<NestedView: HasContextView<T, ContextView = Self::View>>
{
    type View;
}

impl<T, D: HasNestedView<NestedView: HasContextView<T>>> HasNestedContextView<T> for D {
    type View = ContextView<NestedView<D>, T>;
}

impl<T, D> HasView<(Nested, Context<T>)> for D
where
    D: HasNestedContextView<T>,
{
    type View = <D as HasNestedContextView<T>>::View;
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

pub trait HasSaveViews<S: Simulator, A: Analysis>:
    HasView<
        NestedNodeSaveKeyView<S, A>,
        View = <<Self as HasView<NestedNodeBundle>>::View as Save<S, A>>::SaveKey,
    > + HasView<NestedNodeBundle, View: Save<S, A>>
    + HasView<
        NestedTerminalSaveKeyView<S, A>,
        View = <<Self as HasView<NestedTerminalBundle>>::View as Save<S, A>>::SaveKey,
    > + HasView<NestedTerminalBundle, View: Save<S, A>>
{
}

impl<T, S, A> HasSaveViews<S, A> for T
where
    S: Simulator,
    A: Analysis,
    T: HasView<
            NestedNodeBundle,
            View: Save<S, A, SaveKey = <Self as HasView<NestedNodeSaveKeyView<S, A>>>::View>,
        > + HasView<NestedNodeSaveKeyView<S, A>>
        + HasView<
            NestedTerminalBundle,
            View: Save<S, A, SaveKey = <Self as HasView<NestedTerminalSaveKeyView<S, A>>>::View>,
        > + HasView<NestedTerminalSaveKeyView<S, A>>,
{
}

pub struct NestedSaveKey<S, A>(PhantomData<(S, A)>);
pub struct NestedSaved<S, A>(PhantomData<(S, A)>);

impl<S: Simulator, A: Analysis, T: HasNestedView<NestedView: Save<S, A>>>
    HasView<NestedSaveKey<S, A>> for T
{
    type View = crate::simulation::data::SaveKey<NestedView<T>, S, A>;
}

impl<S: Simulator, A: Analysis, T: HasNestedView<NestedView: Save<S, A>>> HasView<NestedSaved<S, A>>
    for T
{
    type View = crate::simulation::data::Saved<NestedView<T>, S, A>;
}

pub struct ContextSaveKey<T, S, A>(PhantomData<(T, S, A)>);
pub struct ContextSaved<T, S, A>(PhantomData<(T, S, A)>);
pub struct NestedContextSaveKey<T, S, A>(PhantomData<(T, S, A)>);
pub struct NestedContextSaved<T, S, A>(PhantomData<(T, S, A)>);

impl<V, S: Simulator, A: Analysis, T: HasContextView<V, ContextView: Save<S, A>>>
    HasView<ContextSaveKey<V, S, A>> for T
{
    type View = crate::simulation::data::SaveKey<ContextView<T, V>, S, A>;
}

impl<V, S: Simulator, A: Analysis, T: HasContextView<V, ContextView: Save<S, A>>>
    HasView<ContextSaved<V, S, A>> for T
{
    type View = crate::simulation::data::Saved<ContextView<T, V>, S, A>;
}

impl<V, S: Simulator, A: Analysis, T: HasNestedContextView<V, View: Save<S, A>>>
    HasView<NestedContextSaveKey<V, S, A>> for T
{
    type View = crate::simulation::data::SaveKey<ContextView<NestedView<T>, V>, S, A>;
}

impl<V, S: Simulator, A: Analysis, T: HasNestedContextView<V, View: Save<S, A>>>
    HasView<NestedContextSaved<V, S, A>> for T
{
    type View = crate::simulation::data::Saved<ContextView<NestedView<T>, V>, S, A>;
}

pub struct NestedNodeSaveKeyView<S, A>(PhantomData<(S, A)>);
pub struct NestedNodeSavedView<S, A>(PhantomData<(S, A)>);
pub struct NestedTerminalSaveKeyView<S, A>(PhantomData<(S, A)>);
pub struct NestedTerminalSavedView<S, A>(PhantomData<(S, A)>);

pub struct SaveKeyView<S, A>(PhantomData<(S, A)>);
pub struct SavedView<S, A>(PhantomData<(S, A)>);

impl<S, A, T> HasView<NestedNodeSaveKeyView<S, A>> for T
where
    S: Simulator,
    A: Analysis,
    T: HasView<NestedNodeBundle>,
    T::View: Save<S, A>,
{
    type View = crate::simulation::data::SaveKey<T::View, S, A>;
}

impl<S, A, T> HasView<NestedNodeSavedView<S, A>> for T
where
    S: Simulator,
    A: Analysis,
    T: HasView<NestedNodeBundle>,
    T::View: Save<S, A>,
{
    type View = crate::simulation::data::Saved<T::View, S, A>;
}

impl<S, A, T> HasView<NestedTerminalSaveKeyView<S, A>> for T
where
    S: Simulator,
    A: Analysis,
    T: HasView<NestedTerminalBundle>,
    T::View: Save<S, A>,
{
    type View = crate::simulation::data::SaveKey<T::View, S, A>;
}

impl<S, A, T> HasView<NestedTerminalSavedView<S, A>> for T
where
    S: Simulator,
    A: Analysis,
    T: HasView<NestedTerminalBundle>,
    T::View: Save<S, A>,
{
    type View = crate::simulation::data::Saved<T::View, S, A>;
}

pub trait HasDefaultLayoutBundle: HasBundleKind {
    type Bundle<S: crate::layout::schema::Schema>: LayoutBundle<S>
        + HasBundleKind<BundleKind = <Self as HasBundleKind>::BundleKind>;
}
/// A port geometry bundle view.
pub struct PortGeometryBundle<S>(PhantomData<S>);

impl<S: crate::layout::schema::Schema, T: HasDefaultLayoutBundle> HasView<PortGeometryBundle<S>>
    for T
{
    type View = T::Bundle<S>;
}

impl<T: HasDefaultLayoutBundle> HasDefaultLayoutBundle for Array<T> {
    type Bundle<S: crate::layout::schema::Schema> = ArrayBundle<T::Bundle<S>>;
}

impl HasDefaultLayoutBundle for Signal {
    type Bundle<S: crate::layout::schema::Schema> = PortGeometry<S::Layer>;
}
