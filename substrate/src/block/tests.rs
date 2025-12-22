use crate::types::TestbenchIo;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use substrate::block::Block;

#[allow(unused)]
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Block)]
#[substrate(io = "TestbenchIo")]
pub struct DelayCellTb<T> {
    pub dut: T,
    pub tr: Decimal,
    pub tf: Decimal,
}
