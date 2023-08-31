use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use substrate::block::{self, Block};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Block)]
#[substrate(io = "substrate::io::TestbenchIo", kind = "block::Cell")]
pub struct DelayCellTb<T> {
    pub dut: T,
    pub tr: Decimal,
    pub tf: Decimal,
}
