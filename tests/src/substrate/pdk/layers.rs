use arcstr::ArcStr;
use substrate::pdk::layers::LayerId;
use substrate::{Layer, Layers};

#[derive(Layers)]
pub struct ExamplePdkLayers {
    #[layer(alias = "met1_drawing", pin = "met1_pin", label = "met1_label")]
    pub met1: Met1,
    #[alias]
    pub met1_drawing: Met1,
    #[layer]
    pub met1_pin: Met1Pin,
    #[layer]
    pub met1_label: Met1Label,
    #[layer]
    pub met2: Met2,
    #[value = "arcstr::literal!(\"test\")"]
    pub global_constant: ArcStr,
}

#[derive(Layer, Clone)]
#[layer(gds = "68/20")]
pub struct Met1 {
    #[id]
    pub id: LayerId,
}

#[derive(Layer)]
#[layer(gds = "68/16")]
pub struct Met1Pin {
    #[id]
    pub id: LayerId,
}

#[derive(Layer)]
#[layer(gds = "68/5")]
pub struct Met1Label {
    #[id]
    pub id: LayerId,
}

#[derive(Layer)]
#[layer(name = "met2", gds = "69/20")]
pub struct Met2 {
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
