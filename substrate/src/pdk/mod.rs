use arcstr::ArcStr;

pub mod example_pdk;

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct LayerId(u64);

#[derive(Debug, Clone, Default)]
pub struct LayerCtx {
    counter: u64,
}

impl AsRef<LayerId> for LayerId {
    #[inline]
    fn as_ref(&self) -> &LayerId {
        self
    }
}
impl AsRef<LayerId> for LayerInfo {
    #[inline]
    fn as_ref(&self) -> &LayerId {
        &self.id
    }
}

impl LayerCtx {
    pub fn new_layer(&mut self) -> LayerId {
        self.counter += 1;
        LayerId(self.counter)
    }

    pub(crate) fn new() -> Self {
        Default::default()
    }
}

pub trait LayerMap {
    fn new(ctx: &mut LayerCtx) -> Self
    where
        Self: Sized;
    fn flatten(&self) -> Vec<LayerInfo>;
    fn unflatten(flat: FlatLayerMap) -> Self
    where
        Self: Sized;
}

pub trait Layer {
    fn info(&self) -> LayerInfo;
    fn from_info(info: LayerInfo) -> Self
    where
        Self: Sized;
}

pub struct FlatLayerMap {
    layers: Vec<LayerInfo>,
}

// Subject to change.
#[derive(Debug, Clone)]
pub struct LayerInfo {
    pub id: LayerId,
    pub name: ArcStr,
    pub gds: Option<(u16, u16)>,
    pub pin: Option<LayerId>,
}

// BEGIN GENERATED
pub struct ExampleLayers {
    pub met1: Met1,
    pub via1: Via1,
    pub met2: Met2,
}

impl LayerMap for ExampleLayers {
    fn new(ctx: &mut LayerCtx) -> Self
    where
        Self: Sized,
    {
        Self {
            met1: Met1 {
                id: ctx.new_layer(),
            },
            via1: Via1 {
                id: ctx.new_layer(),
                my_custom_constant: 130,
            },
            met2: Met2 {
                id: ctx.new_layer(),
            },
        }
    }

    fn flatten(&self) -> Vec<LayerInfo> {
        // TODO: clean this up by making this use `Layer::info`.
        vec![
            LayerInfo {
                id: self.met1.id,
                name: arcstr::literal!("met1"),
                gds: Some((1, 2)),
            },
            LayerInfo {
                id: self.via1.id,
                name: arcstr::literal!("via1"),
                gds: Some((3, 4)),
            },
            LayerInfo {
                id: self.met2.id,
                name: arcstr::literal!("met2"),
                gds: Some((5, 6)),
            },
        ]
    }

    fn unflatten(flat: FlatLayerMap) -> Self
    where
        Self: Sized,
    {
        Self {
            met1: Met1 {
                id: flat.layers[0].id,
            },
            via1: Via1 {
                id: flat.layers[1].id,
                my_custom_constant: 130,
            },
            met2: Met2 {
                id: flat.layers[2].id,
            },
        }
    }
}

pub struct DerivedLayers {
    pub met1: LayerInfo,
    pub via1: LayerInfo,
}

// TODO maybe have separate APIs/traits for derived layermaps?
impl LayerMap for DerivedLayers {
    fn new(_ctx: &mut LayerCtx) -> Self
    where
        Self: Sized,
    {
        panic!("cannot create a new DerivedLayers");
    }

    fn flatten(&self) -> Vec<LayerInfo> {
        vec![self.met1.clone(), self.via1.clone()]
    }

    fn unflatten(flat: FlatLayerMap) -> Self
    where
        Self: Sized,
    {
        // cloning isn't necessary here, but done for brevity
        Self {
            met1: flat.layers[0].clone(),
            via1: flat.layers[1].clone(),
        }
    }
}

impl From<ExampleLayers> for DerivedLayers {
    fn from(value: ExampleLayers) -> Self {
        Self {
            met1: value.met1.info(),
            via1: value.via1.info(),
        }
    }
}

pub struct Met1 {
    pub id: LayerId,
}

pub struct Via1 {
    pub id: LayerId,
    pub my_custom_constant: i64,
}

pub struct Met2 {
    pub id: LayerId,
}

impl Layer for Met1 {
    fn info(&self) -> LayerInfo {
        LayerInfo {
            id: self.id,
            name: arcstr::literal!("met1"),
            gds: Some((1, 2)),
        }
    }

    fn from_info(info: LayerInfo) -> Self
    where
        Self: Sized,
    {
        Self { id: info.id }
    }
}
impl Layer for Via1 {
    fn info(&self) -> LayerInfo {
        LayerInfo {
            id: self.id,
            name: arcstr::literal!("via1"),
            gds: Some((5, 6)),
        }
    }
    fn from_info(info: LayerInfo) -> Self
    where
        Self: Sized,
    {
        Self {
            id: info.id,
            my_custom_constant: 130,
        }
    }
}
impl Layer for Met2 {
    fn info(&self) -> LayerInfo {
        LayerInfo {
            id: self.id,
            name: arcstr::literal!("met2"),
            gds: Some((5, 6)),
        }
    }
    fn from_info(info: LayerInfo) -> Self
    where
        Self: Sized,
    {
        Self { id: info.id }
    }
}

impl AsRef<LayerId> for Met1 {
    #[inline]
    fn as_ref(&self) -> &LayerId {
        &self.id
    }
}
impl AsRef<LayerId> for Via1 {
    #[inline]
    fn as_ref(&self) -> &LayerId {
        &self.id
    }
}
impl AsRef<LayerId> for Met2 {
    #[inline]
    fn as_ref(&self) -> &LayerId {
        &self.id
    }
}

impl ExampleLayers {
    pub fn get_m1(&self) -> &dyn Layer {
        &self.met1
    }
}
// END GENERATED

fn install_layermap() -> ExampleLayers {
    let mut ctx = LayerCtx::new();
    ExampleLayers::new(&mut ctx)
}

#[derive(Debug, Clone, Default)]
struct Builder {
    shapes: Vec<(LayerId, i64)>,
}

impl Builder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn draw_shape(&mut self, layer: impl AsRef<LayerId>, shape: i64) {
        self.shapes.push((*layer.as_ref(), shape));
    }
}

pub fn example_usage() {
    let mut builder = Builder::new();

    let layermap = install_layermap();
    let layermap = DerivedLayers::from(layermap);
    builder.draw_shape(&layermap.met1, 10);
    builder.draw_shape(layermap.via1.id, 20);
    builder.draw_shape(layermap.met1.as_ref(), 30);
    assert_eq!(layermap.met1.gds, Some((1, 2)));
    assert_eq!(layermap.met1.name, "met1");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_usage_works() {
        example_usage();
    }
}
