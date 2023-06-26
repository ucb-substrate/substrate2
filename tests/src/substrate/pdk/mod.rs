use substrate::pdk::layers::Layer;
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

    assert_eq!(ctx.layers.met1a.id, ctx.layers.met1a_drawing.id);
    assert_eq!(ctx.layers.met1a.info().gds, Some((68, 20)));
    assert_eq!(ctx.layers.met1a_drawing.info().gds, Some((68, 20)));
    assert_eq!(ctx.layers.met1a_pin.info().gds, Some((68, 16)));
    assert_eq!(ctx.layers.met1a_label.info().gds, Some((68, 5)));
    assert_eq!(ctx.layers.met2a.info().gds, Some((69, 20)));

    assert_eq!(ctx.layers.global_constant, arcstr::literal!("test"));
    assert_eq!(ctx.layers.met2a.custom_constant, 5);
    assert_eq!(ctx.layers.met2a.more_complex_constant, 5 + 6 + 7);
}
