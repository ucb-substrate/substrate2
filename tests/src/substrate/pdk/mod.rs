use substrate::pdk::layers::{GdsLayerSpec, Layer};
use substrate::{context::Context, pdk::Pdk};

use self::layers::{ExamplePdkALayers, ExamplePdkBLayers};

pub mod layers;

pub struct ExamplePdkA;

impl Pdk for ExamplePdkA {
    type Layers = ExamplePdkALayers;
}

pub struct ExamplePdkB;

impl Pdk for ExamplePdkB {
    type Layers = ExamplePdkBLayers;
}

#[test]
fn test_pdk_layers() {
    let ctx = Context::new(ExamplePdkA);

    assert_eq!(
        ctx.pdk.layers.met1a.as_ref(),
        ctx.pdk.layers.met1a_drawing.as_ref()
    );
    assert_eq!(ctx.pdk.layers.met1a.info().gds, Some(GdsLayerSpec(68, 20)));
    assert_eq!(
        ctx.pdk.layers.met1a_drawing.info().gds,
        Some(GdsLayerSpec(68, 20))
    );
    assert_eq!(
        ctx.pdk.layers.met1a_pin.info().gds,
        Some(GdsLayerSpec(68, 16))
    );
    assert_eq!(
        ctx.pdk.layers.met1a_label.info().gds,
        Some(GdsLayerSpec(68, 5))
    );
    assert_eq!(ctx.pdk.layers.met2a.info().gds, Some(GdsLayerSpec(69, 20)));

    assert_eq!(ctx.pdk.layers.polya.custom_property(), 5)
}
