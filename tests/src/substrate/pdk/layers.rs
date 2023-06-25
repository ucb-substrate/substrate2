use arcstr::ArcStr;
use substrate::pdk::layers::LayerId;

#[derive(Layers)]
pub struct ExamplePdkALayers {
    #[layer(alias = "met1_drawing", pin = "met1_pin", label = "met1_label")]
    pub met1: Met1,
    #[layer]
    pub met1_pin: Met1Pin,
    #[layer]
    pub met1_label: Met1Label,
    #[layer]
    pub met2: Met2,
    #[value = "arcstr::literal!(\"test\")"]
    pub global_constant: ArcStr,
}

#[derive(Layer)]
#[layer(gds = "66/20")]
pub struct Met1 {
    pub id: LayerId,
}

#[derive(Layer)]
#[layer(name = "met2", gds = "66/20")]
pub struct Met2 {
    pub id: LayerId,
    #[value = "5"]
    pub custom_constant: u64,
    #[value = "compute_constant()"]
    pub more_complex_constant: u64,
}

fn compute_constant() -> u64 {
    5 + 6 + 7
}
