use substrate::{
    context::Context,
    pdk::layers::{GdsLayerSpec, Layer},
};

use crate::shared::pdk::{ExamplePdkA, NmosA};

#[test]
fn test_pdk_layers() {
    let ctx = Context::new(ExamplePdkA);

    assert_eq!(
        ctx.pdk.layers.met1a.drawing.info().gds,
        Some(GdsLayerSpec(68, 20))
    );
    assert_eq!(
        ctx.pdk.layers.met1a.pin.info().gds,
        Some(GdsLayerSpec(68, 16))
    );
    assert_eq!(
        ctx.pdk.layers.met1a.label.info().gds,
        Some(GdsLayerSpec(68, 5))
    );
    assert_eq!(ctx.pdk.layers.met2a.info().gds, Some(GdsLayerSpec(69, 20)));

    assert_eq!(ctx.pdk.layers.polya.custom_property(), 5)
}

#[test]
fn export_nmos_a() {
    let mut ctx = Context::new(ExamplePdkA);
    let lib = ctx.export_scir(NmosA { w: 1_200, l: 150 });
    assert_eq!(lib.cells().count(), 1);
    let issues = lib.validate();
    println!("Library:\n{:#?}", lib);
    println!("Issues = {:#?}", issues);
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);

    let mos = lib.cell_named("nmos_a_w1200_l150");
    assert_eq!(mos.ports().count(), 4);
    assert_eq!(mos.primitives().count(), 1);
    assert_eq!(mos.instances().count(), 0);
}
