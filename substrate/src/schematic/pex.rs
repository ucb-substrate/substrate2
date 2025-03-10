//! Utilities for tracking nested data through parasitic extraction.

use std::sync::Arc;

use scir::{Library, NamedSliceOne, NetlistLibConversion, SliceOnePath};

use crate::{
    simulation::{
        data::{Save, SaveKey, Saved},
        Analysis, Simulator,
    },
    types::schematic::{NestedNode, RawNestedNode},
};

use super::{
    conv::{ConvertedNodePath, RawLib},
    schema::Schema,
    Cell, ContextView, HasContextView, HasNestedView, InstancePath, NestedView, Schematic,
};

/// Captures information for mapping nodes/elements between schematic and extracted netlists.
pub struct PexContext<S: Schema> {
    /// The source spice file for this DSPF extracted view.
    lib: Arc<RawLib<S>>,
    conv: Arc<NetlistLibConversion>,
    path: InstancePath,
}

impl<S: Schema> Clone for PexContext<S> {
    fn clone(&self) -> Self {
        Self {
            lib: self.lib.clone(),
            conv: self.conv.clone(),
            path: self.path.clone(),
        }
    }
}

/// A schema that can convert element paths to strings.
pub trait StringPathSchema: Schema {
    /// Convert a node path to a raw string.
    fn node_path(lib: &Library<Self>, conv: &NetlistLibConversion, path: &SliceOnePath) -> String;
}

impl<S: StringPathSchema> HasContextView<PexContext<S>> for NestedNode {
    type ContextView = RawNestedNode;

    fn context_view(&self, parent: &PexContext<S>) -> ContextView<Self, PexContext<S>> {
        let n = self;
        let path = parent.lib.convert_node_path(&n.path()).unwrap();
        let path = match path {
            ConvertedNodePath::Cell(path) => path,
            ConvertedNodePath::Primitive {
                instances, port, ..
            } => SliceOnePath::new(instances.clone(), NamedSliceOne::new(port.clone())),
        };
        let path = parent.lib.scir.simplify_path(path);
        RawNestedNode::new(
            parent.path.clone(),
            S::node_path(&parent.lib.scir, &parent.conv, &path),
        )
    }
}

/// Nested data exposed by an extracted view of a circuit.
pub struct PexData<T: Schematic> {
    cell: Cell<Arc<T>>,
    lib: Arc<RawLib<T::Schema>>,
    conv: Arc<NetlistLibConversion>,
}

impl<T: Schematic> Clone for PexData<T> {
    fn clone(&self) -> Self {
        Self {
            cell: self.cell.clone(),
            lib: self.lib.clone(),
            conv: self.conv.clone(),
        }
    }
}

impl<T: Schematic> PexData<T> {
    /// Creates a new [`PexData`].
    pub fn new(
        cell: Cell<Arc<T>>,
        lib: Arc<RawLib<T::Schema>>,
        conv: Arc<NetlistLibConversion>,
    ) -> Self {
        Self { cell, lib, conv }
    }
}

/// The nested view of [`PexData`].
pub struct NestedPexData<T: Schematic> {
    cell: Cell<Arc<T>>,
    ctx: PexContext<T::Schema>,
}

impl<T: Schematic> Clone for NestedPexData<T> {
    fn clone(&self) -> Self {
        Self {
            cell: self.cell.clone(),
            ctx: self.ctx.clone(),
        }
    }
}

impl<T: Schematic> NestedPexData<T>
where
    T::NestedData: HasContextView<PexContext<T::Schema>>,
{
    /// Access the underlying data.
    pub fn data(&self) -> ContextView<T::NestedData, PexContext<T::Schema>> {
        self.cell.context_data(&self.ctx)
    }
}

impl<T: Schematic> HasNestedView for PexData<T> {
    type NestedView = NestedPexData<T>;
    fn nested_view(&self, parent: &InstancePath) -> NestedView<Self> {
        NestedPexData {
            cell: self.cell.clone(),
            ctx: PexContext {
                lib: self.lib.clone(),
                conv: self.conv.clone(),
                path: parent.clone(),
            },
        }
    }
}

impl<T: Schematic> HasNestedView for NestedPexData<T> {
    type NestedView = NestedPexData<T>;
    fn nested_view(&self, parent: &InstancePath) -> NestedView<Self> {
        NestedPexData {
            cell: self.cell.clone(),
            ctx: PexContext {
                lib: self.ctx.lib.clone(),
                conv: self.ctx.conv.clone(),
                path: self.ctx.path.prepend(parent),
            },
        }
    }
}

impl<S: Simulator, A: Analysis, T: Schematic> Save<S, A> for NestedPexData<T>
where
    T::NestedData: HasContextView<PexContext<T::Schema>>,
    ContextView<T::NestedData, PexContext<T::Schema>>: Save<S, A>,
{
    type SaveKey = SaveKey<ContextView<T::NestedData, PexContext<T::Schema>>, S, A>;
    type Saved = Saved<ContextView<T::NestedData, PexContext<T::Schema>>, S, A>;

    fn save(
        &self,
        ctx: &substrate::simulation::SimulationContext<S>,
        opts: &mut <S as Simulator>::Options,
    ) -> <Self as Save<S, A>>::SaveKey {
        self.data().save(ctx, opts)
    }

    fn from_saved(
        output: &<A as Analysis>::Output,
        key: &<Self as Save<S, A>>::SaveKey,
    ) -> <Self as Save<S, A>>::Saved {
        <ContextView<T::NestedData, PexContext<T::Schema>> as Save<S, A>>::from_saved(output, key)
    }
}
