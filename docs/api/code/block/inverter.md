#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Inverter {
    strength: usize,
}
impl Inverter {
    pub fn new(strength: usize) -> Self {
        Self { strength }
    }
}
impl Block for Inverter {
    fn id() -> arcstr::ArcStr {
        arcstr::literal!("inverter")
    }
    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("inverter_{}", self.strength)
    }
}
