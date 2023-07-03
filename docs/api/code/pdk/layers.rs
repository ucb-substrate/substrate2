#[derive(Layers)]
pub struct ExamplePdkLayers {
    #[layer(gds = "66/20")]
    pub poly: Poly,
    #[layer_family]
    pub met1: Met1,
    #[layer(name = "met2", gds = "69/20")]
    pub met2: Met2,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Met1 {
    #[layer(gds = "68/20", primary)]
    pub met1_drawing: Met1Drawing,
    #[layer(gds = "68/16", pin)]
    pub met1_pin: Met1Pin,
    #[layer(gds = "68/5", label)]
    pub met1_label: Met1Label,
}