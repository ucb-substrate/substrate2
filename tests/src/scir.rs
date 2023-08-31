use crate::netlist::vdivider;
use scir::{Library, LibraryBuilder};
use spice::Primitive;

#[test]
fn merge_scir_libraries() {
    let mut lib1: LibraryBuilder<Primitive> = (*vdivider()).clone();
    let lib2: Library<Primitive> = vdivider();
    let mapping = lib1.merge(&lib2);

    let preserved_id = lib1.cell_id_named("vdivider");
    let old_id = lib2.cell_id_named("vdivider");
    let new_id = mapping.new_cell_id(old_id);

    let issues = lib1.validate();
    assert_eq!(issues.num_warnings(), 0);
    assert_eq!(issues.num_errors(), 0);

    assert_eq!(lib1.cells().count(), 4);

    let new_name = lib1.cell(new_id).name();

    assert_ne!(new_name, "vdivider");
    assert!(new_name.starts_with("vdivider"));
    assert_eq!(lib1.cell(preserved_id).name(), "vdivider");
}
