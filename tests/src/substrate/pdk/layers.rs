use substrate::Layers;

#[derive(Layers)]
pub struct ExamplePdkALayers {
    #[layer(gds = "66/20")]
    pub polya: PolyA,
    #[layer(gds = "68/20")]
    #[pin(pin = "met1a_pin", label = "met1a_label")]
    pub met1a: Met1A,
    #[layer(alias = "met1a")]
    pub met1a_drawing: Met1A,
    #[layer(gds = "68/16")]
    pub met1a_pin: Met1PinA,
    #[layer(gds = "68/5")]
    pub met1a_label: Met1LabelA,
    #[layer(name = "met2", gds = "69/20")]
    pub met2a: Met2A,
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
    #[pin(pin = "met1b_pin", label = "met1b_label")]
    pub met1b: Met1B,
    #[layer(alias = "met1b")]
    pub met1b_drawing: Met1B,
    #[layer(gds = "15/15")]
    pub met1b_pin: Met1PinB,
    #[layer(gds = "15/2")]
    pub met1b_label: Met1LabelB,
    #[layer(name = "met2", gds = "16/30")]
    pub met2b: Met2B,
}
