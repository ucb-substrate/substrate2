use substrate::pdk::layers::Layer;
use substrate::{context::Context, pdk::Pdk};

use self::layers::ExamplePdkLayers;

pub mod layers;

pub struct ExamplePdkA;

impl Pdk for ExamplePdkA {
    type Layers = ExamplePdkLayers;
}

pub struct ExamplePdkB;

impl Pdk for ExamplePdkB {
    type Layers = ExamplePdkLayers;
}

#[test]
fn test_pdk_layers() {
    let ctx = Context::new(ExamplePdkA);

    assert_eq!(ctx.layers.met1.id, ctx.layers.met1_drawing.id);
    assert_eq!(ctx.layers.met1.info().gds, Some((68, 20)));
    assert_eq!(ctx.layers.met1_drawing.info().gds, Some((68, 20)));
    assert_eq!(ctx.layers.met1_pin.info().gds, Some((68, 16)));
    assert_eq!(ctx.layers.met1_label.info().gds, Some((68, 5)));
    assert_eq!(ctx.layers.met2.info().gds, Some((69, 20)));

    assert_eq!(ctx.layers.global_constant, arcstr::literal!("test"));
    assert_eq!(ctx.layers.met2.custom_constant, 5);
    assert_eq!(ctx.layers.met2.more_complex_constant, 5 + 6 + 7);
}
