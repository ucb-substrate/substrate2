use arcstr::ArcStr;
use substrate::pdk::layers::LayerId;
use substrate::{Layer, Layers};

#[derive(Layers)]
pub struct ExamplePdkALayers {
    #[layer]
    pub polya: PolyA,
    #[layer(alias = "met1a_drawing", pin = "met1a_pin", label = "met1a_label")]
    pub met1a: Met1A,
    #[alias]
    pub met1a_drawing: Met1A,
    #[layer]
    pub met1a_pin: Met1PinA,
    #[layer]
    pub met1a_label: Met1LabelA,
    #[layer]
    pub met2a: Met2A,
    #[value = "arcstr::literal!(\"test\")"]
    pub global_constant: ArcStr,
}

#[derive(Layer)]
#[layer(gds = "66/20")]
pub struct PolyA {
    #[id]
    pub id: LayerId,
}

#[derive(Layer, Clone)]
#[layer(gds = "68/20")]
pub struct Met1A {
    #[id]
    pub id: LayerId,
}

#[derive(Layer)]
#[layer(gds = "68/16")]
pub struct Met1PinA {
    #[id]
    pub id: LayerId,
}

#[derive(Layer)]
#[layer(gds = "68/5")]
pub struct Met1LabelA {
    #[id]
    pub id: LayerId,
}

#[derive(Layer)]
#[layer(name = "met2", gds = "69/20")]
pub struct Met2A {
    #[id]
    pub id: LayerId,
    #[value = "5"]
    pub custom_constant: u64,
    #[value = "compute_constant()"]
    pub more_complex_constant: u64,
}

fn compute_constant() -> u64 {
    5 + 6 + 7
}

#[derive(Layers)]
pub struct ExamplePdkBLayers {
    #[layer]
    pub polyb: PolyB,
    #[layer(alias = "met1b_drawing", pin = "met1b_pin", label = "met1b_label")]
    pub met1b: Met1B,
    #[alias]
    pub met1b_drawing: Met1B,
    #[layer]
    pub met1b_pin: Met1PinB,
    #[layer]
    pub met1b_label: Met1LabelB,
    #[layer]
    pub met2b: Met2B,
}

#[derive(Layer, Clone)]
#[layer(gds = "13/30")]
pub struct PolyB {
    #[id]
    pub id: LayerId,
}

#[derive(Layer, Clone)]
#[layer(gds = "15/30")]
pub struct Met1B {
    #[id]
    pub id: LayerId,
}

#[derive(Layer)]
#[layer(gds = "15/15")]
pub struct Met1PinB {
    #[id]
    pub id: LayerId,
}

#[derive(Layer)]
#[layer(gds = "15/2")]
pub struct Met1LabelB {
    #[id]
    pub id: LayerId,
}

#[derive(Layer)]
#[layer(name = "met2", gds = "16/30")]
pub struct Met2B {
    #[id]
    pub id: LayerId,
}
