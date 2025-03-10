//! Interfaces for interacting with simulation data.

use std::ops::Deref;

use codegen::impl_save_tuples;

use crate::{
    schematic::{HasNestedView, NestedInstance, NestedView, Schematic},
    simulation::{Analysis, SimulationContext, Simulator},
    types::{
        schematic::{IoTerminalBundle, NestedNode},
        ArrayBundle, HasBundleKind,
    },
};

/// Saves the raw output of a simulation.
#[derive(Debug, Clone, Copy)]
pub struct SaveOutput;

/// Saves the transient time waveform.
#[derive(Debug, Clone, Copy)]
pub struct SaveTime;

/// Saves the frequency vector of an AC (frequency sweep) simulation.
#[derive(Debug, Clone, Copy)]
pub struct SaveFreq;

impl HasNestedView for SaveOutput {
    type NestedView = SaveOutput;

    fn nested_view(&self, _parent: &substrate::schematic::InstancePath) -> Self::NestedView {
        *self
    }
}

impl HasNestedView for SaveTime {
    type NestedView = SaveTime;

    fn nested_view(&self, _parent: &substrate::schematic::InstancePath) -> Self::NestedView {
        *self
    }
}

/// Gets the [`Save::SaveKey`] corresponding to type `T`.
pub type SaveKey<T, S, A> = <T as Save<S, A>>::SaveKey;

/// Gets the [`Save::Saved`] corresponding to type `T`.
pub type Saved<T, S, A> = <T as Save<S, A>>::Saved;

/// A schematic object that can be saved in an analysis within a given simulator.
pub trait Save<S: Simulator, A: Analysis> {
    /// The key type used to address the saved output within the analysis.
    ///
    /// This key is assigned in [`Save::save`].
    type SaveKey;
    /// The saved data associated with things object.
    type Saved;

    /// Marks the given output for saving, returning a key that can be used to recover
    /// the output once the simulation is complete.
    fn save(
        &self,
        ctx: &SimulationContext<S>,
        opts: &mut <S as Simulator>::Options,
    ) -> <Self as Save<S, A>>::SaveKey;

    /// Recovers the desired simulation output from the analysis's output.
    fn from_saved(
        output: &<A as Analysis>::Output,
        key: &<Self as Save<S, A>>::SaveKey,
    ) -> <Self as Save<S, A>>::Saved;
}

impl_save_tuples! {64, NestedNode}

impl<T: Save<S, A>, S: Simulator, A: Analysis> Save<S, A> for Option<T> {
    type SaveKey = Option<SaveKey<T, S, A>>;
    type Saved = Option<Saved<T, S, A>>;

    fn save(
        &self,
        ctx: &SimulationContext<S>,
        opts: &mut <S as Simulator>::Options,
    ) -> <Self as Save<S, A>>::SaveKey {
        self.as_ref().map(|x| x.save(ctx, opts))
    }

    fn from_saved(
        output: &<A as Analysis>::Output,
        key: &<Self as Save<S, A>>::SaveKey,
    ) -> <Self as Save<S, A>>::Saved {
        key.as_ref().map(|k| T::from_saved(output, k))
    }
}

impl<T: Save<S, A>, S: Simulator, A: Analysis> Save<S, A> for Vec<T> {
    type SaveKey = Vec<SaveKey<T, S, A>>;
    type Saved = Vec<Saved<T, S, A>>;

    fn save(
        &self,
        ctx: &SimulationContext<S>,
        opts: &mut <S as Simulator>::Options,
    ) -> <Self as Save<S, A>>::SaveKey {
        self.iter().map(|x| x.save(ctx, opts)).collect()
    }

    fn from_saved(
        output: &<A as Analysis>::Output,
        key: &<Self as Save<S, A>>::SaveKey,
    ) -> <Self as Save<S, A>>::Saved {
        key.iter().map(|k| T::from_saved(output, k)).collect()
    }
}

impl<T: HasBundleKind + Save<S, A>, S: Simulator, A: Analysis> Save<S, A> for ArrayBundle<T> {
    type SaveKey = Vec<SaveKey<T, S, A>>;
    type Saved = Vec<Saved<T, S, A>>;

    fn save(
        &self,
        ctx: &SimulationContext<S>,
        opts: &mut <S as Simulator>::Options,
    ) -> <Self as Save<S, A>>::SaveKey {
        (0..self.num_elems())
            .map(|i| self[i].save(ctx, opts))
            .collect()
    }

    fn from_saved(
        output: &<A as Analysis>::Output,
        key: &<Self as Save<S, A>>::SaveKey,
    ) -> <Self as Save<S, A>>::Saved {
        <Vec<T>>::from_saved(output, key)
    }
}

/// The result of saving a nested instance in a simulation.
///
/// Saves the nested instance's data and IO.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct NestedInstanceOutput<D, I> {
    /// The nested data of the instance.
    data: D,
    /// The IO of the instance.
    io: I,
}

impl<D, I> Deref for NestedInstanceOutput<D, I> {
    type Target = D;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<D, I> NestedInstanceOutput<D, I> {
    /// The ports of this instance.
    pub fn io(&self) -> &I {
        &self.io
    }

    /// The nested data.
    pub fn data(&self) -> &D {
        &self.data
    }
}

impl<S: Simulator, A: Analysis> Save<S, A> for () {
    type SaveKey = ();
    type Saved = ();

    fn save(
        &self,
        _ctx: &SimulationContext<S>,
        _opts: &mut <S as Simulator>::Options,
    ) -> <Self as Save<S, A>>::SaveKey {
    }

    fn from_saved(
        _output: &<A as Analysis>::Output,
        _key: &<Self as Save<S, A>>::SaveKey,
    ) -> <Self as Save<S, A>>::Saved {
    }
}

impl<T, S, A> Save<S, A> for NestedInstance<T>
where
    T: Schematic,
    S: Simulator,
    A: Analysis,
    NestedView<T::NestedData>: Save<S, A>,
    NestedView<IoTerminalBundle<T>>: Save<S, A>,
{
    type SaveKey = (
        <NestedView<T::NestedData> as Save<S, A>>::SaveKey,
        <NestedView<IoTerminalBundle<T>> as Save<S, A>>::SaveKey,
    );
    type Saved = NestedInstanceOutput<
        <NestedView<T::NestedData> as Save<S, A>>::Saved,
        <NestedView<IoTerminalBundle<T>> as Save<S, A>>::Saved,
    >;

    fn save(
        &self,
        ctx: &SimulationContext<S>,
        opts: &mut <S as Simulator>::Options,
    ) -> <Self as Save<S, A>>::SaveKey {
        let data = self.data().save(ctx, opts);
        let io = self.io().save(ctx, opts);
        (data, io)
    }

    fn from_saved(
        output: &<A as Analysis>::Output,
        key: &<Self as Save<S, A>>::SaveKey,
    ) -> <Self as Save<S, A>>::Saved {
        NestedInstanceOutput {
            data: <NestedView<T::NestedData> as Save<S, A>>::from_saved(output, &key.0),
            io: <NestedView<IoTerminalBundle<T>> as Save<S, A>>::from_saved(output, &key.1),
        }
    }
}
