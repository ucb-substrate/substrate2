use substrate::{
    context::PdkContext,
    pdk::layers::{GdsLayerSpec, Layer},
    schematic::conv::RawLib,
};

use crate::shared::pdk::{ExamplePdkA, NmosA};

#[test]
fn pdk_layers() {
    let ctx = PdkContext::new(ExamplePdkA);

    assert_eq!(
        ctx.layers.met1a.drawing.info().gds,
        Some(GdsLayerSpec(68, 20))
    );
    assert_eq!(ctx.layers.met1a.pin.info().gds, Some(GdsLayerSpec(68, 16)));
    assert_eq!(ctx.layers.met1a.label.info().gds, Some(GdsLayerSpec(68, 5)));
    assert_eq!(ctx.layers.met2a.info().gds, Some(GdsLayerSpec(69, 20)));

    assert_eq!(ctx.layers.polya.custom_property(), 5)
}

#[test]
fn export_nmos_a() {
    let ctx = PdkContext::new(ExamplePdkA);
    let RawLib { scir, conv: _ } = ctx
        .export_scir::<ExamplePdkA, _>(NmosA { w: 1_200, l: 150 })
        .unwrap();
    assert_eq!(scir.primitives().count(), 1);
    let issues = scir.validate();
    println!("Library:\n{:#?}", scir);
    println!("Issues = {:#?}", issues);
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);
}
