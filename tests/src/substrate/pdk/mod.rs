use substrate::pdk::Pdk;

pub mod layers;

pub struct ExamplePdkA;

impl Pdk for ExamplePdkA {}

pub struct ExamplePdkB;

impl Pdk for ExamplePdkB {}
