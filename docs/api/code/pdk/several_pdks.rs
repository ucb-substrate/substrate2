pub struct ExamplePdkA;

impl Pdk for ExamplePdkA {
    type Layers = ExamplePdkALayers;
    type Corner = ExampleCorner;
}

pub struct ExamplePdkB;

impl Pdk for ExamplePdkB {
    type Layers = ExamplePdkBLayers;
    type Corner = ExampleCorner;
}

#[derive(Debug, Clone, Copy)]
pub struct ExampleCorner;

impl Corner for ExampleCorner {
    fn name(&self) -> arcstr::ArcStr {
        arcstr::literal!("example_corner")
    }
}
