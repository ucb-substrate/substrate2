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
