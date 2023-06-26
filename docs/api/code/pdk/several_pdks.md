pub struct ExamplePdkA;

impl Pdk for ExamplePdkA {
    type Layers = ExamplePdkLayers;
}

pub struct ExamplePdkB;

impl Pdk for ExamplePdkB {
    type Layers = ExamplePdkLayers;
}

