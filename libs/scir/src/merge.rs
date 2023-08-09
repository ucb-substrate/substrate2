//! Merge SCIR libraries.

use uniquify::Names;

use super::*;

/// Keeps track of cell IDs after a library is merged.
pub struct MergedMapping {
    cells: HashMap<CellId, CellId>,
}

struct Merger<'a> {
    /// Source cell ID -> destination cell ID
    mapping: HashMap<CellId, CellId>,
    /// Destination cell ID -> name.
    names: Names<CellId>,
    dst: &'a mut LibraryBuilder,
    src: &'a LibraryBuilder,
}

impl<'a> Merger<'a> {
    #[inline]
    fn new(dst: &'a mut LibraryBuilder, src: &'a LibraryBuilder) -> Self {
        Self {
            mapping: HashMap::with_capacity(src.cells.len()),
            names: Names::with_capacity(src.cells.len() + dst.cells.len()),
            dst,
            src,
        }
    }

    fn merge(mut self) -> MergedMapping {
        for (id, cell) in self.dst.cells() {
            self.names.reserve_name(id, cell.name());
        }
        for (id, cell) in self.src.cells() {
            self.merge_cell(id, cell);
        }

        MergedMapping {
            cells: self.mapping,
        }
    }

    fn merge_cell(&mut self, id: CellId, cell: &Cell) -> CellId {
        if let Some(&id) = self.mapping.get(&id) {
            return id;
        }

        let n_id = self.dst.alloc_id();
        let n_name = self.names.assign_name(n_id, &cell.name);
        self.mapping.insert(id, n_id);
        let mut n_cell = cell.clone();
        n_cell.name = n_name;

        if let Opacity::Clear(inner) = n_cell.contents_mut() {
            for (_, inst) in inner.instances_mut() {
                let n_child_id = self.merge_cell(inst.cell(), self.src.cell(inst.cell()));
                inst.cell = n_child_id;
            }
        }

        self.dst.add_cell_with_id(n_id, n_cell);
        n_id
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
}

impl LibraryBuilder {
    /// Merges another SCIR library into the current library.
    pub fn merge(&mut self, other: &Self) -> MergedMapping {
        Merger::new(self, other).merge()
    }
}
