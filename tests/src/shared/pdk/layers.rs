use substrate::pdk::layers::{LayerFamily, Layers};

#[derive(Layers)]
pub struct ExamplePdkALayers {
    #[layer(gds = "66/20")]
    pub polya: PolyA,
    #[layer_family]
    pub met1a: Met1A,
    #[layer(name = "met2", gds = "69/20")]
    pub met2a: Met2A,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Met1A {
    #[layer(gds = "68/20", primary)]
    pub drawing: Met1DrawingA,
    #[layer(gds = "68/16", pin)]
    pub pin: Met1PinA,
    #[layer(gds = "68/5", label)]
    pub label: Met1LabelA,
}

impl PolyA {
    pub const fn custom_property(&self) -> u64 {
        5
    }
}

#[derive(Layers)]
pub struct ExamplePdkBLayers {
    #[layer(gds = "13/30")]
    pub polyb: PolyB,
    #[layer(gds = "15/30")]
    pub met1b: Met1B,
    #[layer(gds = "15/15")]
    pub met1b_pin: Met1PinB,
    #[layer(gds = "15/2")]
    pub met1b_label: Met1LabelB,
    #[layer(name = "met2", gds = "16/30")]
    pub met2b: Met2B,
}
