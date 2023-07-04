pub struct ExamplePdkA;

impl Pdk for ExamplePdkA {
    type Layers = ExamplePdkALayers;
    type Corner = ExampleCorner;
    fn corner(&self, name: &str) -> Option<Self::Corner> {
        match name {
            "example_corner" => Some(ExampleCorner),
            _ => None,
        }
    }
}

pub struct ExamplePdkB;

impl Pdk for ExamplePdkB {
    type Layers = ExamplePdkBLayers;
    type Corner = ExampleCorner;
    fn corner(&self, name: &str) -> Option<Self::Corner> {
        match name {
            "example_corner" => Some(ExampleCorner),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ExampleCorner;

impl Corner for ExampleCorner {
    fn name(&self) -> arcstr::ArcStr {
        arcstr::literal!("example_corner")
    }
}
