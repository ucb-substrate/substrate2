use super::*;

// BEGIN GENERATED FROM example_pdk.toml
pub struct ExamplePdk {
    layers: ExamplePdkLayers,
}

pub struct ExamplePdkLayers {
    met1: Met1,
    met1_drawing: Met1Drawing,
}

pub struct Met1 {
    primary: LayerId,
    pin: LayerId,
}

pub struct Met1Drawing(LayerId);

impl Layer for Met1 {
    fn info(&self) -> super::LayerInfo {
        LayerInfo {
            id: self.primary,
            name: arcstr::literal!("met1"),
            gds: Some((1, 2)),
            pin: None,
        }
    }

    fn from_info(info: LayerInfo) -> Self {
        Self {
            primary: info.id,
            pin: info.pin.unwrap(),
        }
    }
}
impl Layer for Met1Drawing {
    fn info(&self) -> super::LayerInfo {
        LayerInfo {
            id: self.0,
            name: arcstr::literal!("met1_drawing"),
            gds: Some((1, 2)),
            pin: None,
        }
    }

    fn from_info(info: LayerInfo) -> Self {
        Self(info.id)
    }
}

impl LayerMap for ExamplePdkLayers {
    fn new(ctx: &mut LayerCtx) -> Self
    where
        Self: Sized,
    {
        let met1_drawing = ctx.new_layer();
        let met1_pin = ctx.new_layer();
        Self {
            met1: Met1 {
                primary: met1_drawing,
                pin: met1_pin,
            },
            met1_drawing: Met1Drawing(met1_drawing),
        }
    }

    fn flatten(&self) -> Vec<LayerInfo> {
        vec![self.met1.info()]
    }

    fn unflatten(flat: FlatLayerMap) -> Self {
        Self {
            met1: Met1::from_info(flat.layers[0].clone()),
            met1_drawing: Met1Drawing::from_info(flat.layers[0].clone()),
        }
    }
}
// END GENERATED
