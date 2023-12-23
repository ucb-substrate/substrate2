//! Merge SCIR libraries.

use uniquify::Names;

use super::*;

/// Keeps track of cell and primitive IDs after a library is merged.
pub struct MergedMapping {
    cells: HashMap<CellId, CellId>,
    primitives: HashMap<PrimitiveId, PrimitiveId>,
}

struct Merger<'a, S: Schema + ?Sized> {
    /// Source cell ID -> destination cell ID
    cell_mapping: HashMap<CellId, CellId>,
    /// Source primitive ID -> destination primitive ID
    primitive_mapping: HashMap<PrimitiveId, PrimitiveId>,
    /// Destination cell ID -> name.
    names: Names<CellId>,
    dst: &'a mut LibraryBuilder<S>,
    src: LibraryBuilder<S>,
}

impl<'a, S: Schema + ?Sized> Merger<'a, S> {
    #[inline]
    fn new(dst: &'a mut LibraryBuilder<S>, src: LibraryBuilder<S>) -> Self {
        Self {
            cell_mapping: HashMap::with_capacity(src.cells.len()),
            primitive_mapping: HashMap::with_capacity(src.primitives.len()),
            names: Names::with_capacity(src.cells.len() + dst.cells.len()),
            dst,
            src,
        }
    }

    fn merge(mut self) -> MergedMapping {
        for (id, cell) in self.dst.cells() {
            self.names.reserve_name(id, cell.name());
        }
        let mut cells: Vec<_> = self.src.cells.drain(..).collect();
        let primitives: Vec<_> = self.src.primitives.drain(..).collect();
        for (id, cell) in cells.iter_mut() {
            self.assign_cell_identifiers(*id, cell);
        }
        for (id, _) in primitives.iter() {
            self.assign_primitive_identifiers(*id);
        }
        for (id, cell) in cells {
            self.merge_cell(id, cell);
        }
        for (id, primitive) in primitives {
            self.merge_primitive(id, primitive);
        }

        MergedMapping {
            cells: self.cell_mapping,
            primitives: self.primitive_mapping,
        }
    }

    fn assign_cell_identifiers(&mut self, id: CellId, cell: &mut Cell) {
        let n_id = self.dst.alloc_cell_id();
        let n_name = self.names.assign_name(n_id, &cell.name);
        self.cell_mapping.insert(id, n_id);
        cell.name = n_name;
    }

    fn assign_primitive_identifiers(&mut self, id: PrimitiveId) {
        let n_id = self.dst.alloc_primitive_id();
        self.primitive_mapping.insert(id, n_id);
    }

    fn merge_cell(&mut self, id: CellId, mut cell: Cell) {
        for (_, inst) in cell.instances.iter_mut() {
            match inst.child() {
                ChildId::Cell(c) => {
                    inst.child = (*self.cell_mapping.get(&c).unwrap()).into();
                }
                ChildId::Primitive(p) => {
                    inst.child = (*self.primitive_mapping.get(&p).unwrap()).into();
                }
            }
        }
        let n_id = self.cell_mapping.get(&id).unwrap();
        self.dst.add_cell_with_id(*n_id, cell);
    }

    fn merge_primitive(&mut self, id: PrimitiveId, primitive: S::Primitive) {
        let n_id = self.primitive_mapping.get(&id).unwrap();
        self.dst.add_primitive_with_id(*n_id, primitive);
    }
}

impl MergedMapping {
    /// Get the cell ID in the merged library
    /// corresponding to `old_cell_id` in the original library.
    ///
    /// # Panics
    ///
    /// Panics if `old_cell_id` is not a valid cell ID
    /// in the original library.
    pub fn new_cell_id(&self, old_cell_id: CellId) -> CellId {
        self.cells.get(&old_cell_id).copied().unwrap()
    }
    /// Get the primitive ID in the merged library
    /// corresponding to `old_primitive_id` in the original library.
    ///
    /// # Panics
    ///
    /// Panics if `old_primitive_id` is not a valid primitive ID
    /// in the original library.
    pub fn new_primitive_id(&self, old_primitive_id: PrimitiveId) -> PrimitiveId {
        self.primitives.get(&old_primitive_id).copied().unwrap()
    }
}

impl<S: Schema + ?Sized> LibraryBuilder<S> {
    /// Merges another SCIR library into the current library.
    pub fn merge(&mut self, other: Self) -> MergedMapping {
        Merger::new(self, other).merge()
    }
}
