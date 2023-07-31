pub struct ExamplePdk;

impl Pdk for ExamplePdk {
    type Layers = ExamplePdkLayers;
    type Corner = ExamplePdkCorner;
}


#[derive(Debug, Clone, Copy)]
pub enum ExamplePdkCorner {
    Tt,
    Ss,
    Ff,
}

impl Corner for ExamplePdkCorner {
    fn name(&self) -> arcstr::ArcStr {
        match *self {
            Self::Tt => arcstr::literal!("tt"),
            Self::Ff => arcstr::literal!("ff"),
            Self::Ss => arcstr::literal!("ss"),
        }
    }
}