use crate::netlist::vdivider;
use spice::Spice;

#[test]
fn merge_scir_libraries() {
    let mut lib1 = (*vdivider::<Spice>()).clone();
    let lib2 = vdivider::<Spice>();
    let mapping = lib1.merge(lib2.clone().into_builder());

    let preserved_id = lib1.cell_id_named("vdivider");
    let old_id = lib2.cell_id_named("vdivider");
    let new_id = mapping.new_cell_id(old_id);

    let issues = lib1.validate();
    assert_eq!(issues.num_warnings(), 0);
    assert_eq!(issues.num_errors(), 0);

    assert_eq!(lib1.cells().count(), 2);

    let new_name = lib1.cell(new_id).name();

    assert_ne!(new_name, "vdivider");
    assert!(new_name.starts_with("vdivider"));
    assert_eq!(lib1.cell(preserved_id).name(), "vdivider");
}
