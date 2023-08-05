use spectre::tran::{TranCurrent, TranVoltage};
use substrate::FromSaved;

#[derive(Debug, Clone, FromSaved)]
#[allow(unused)]
pub enum SavedEnum {
    Fields {
        vout: TranVoltage,
        iout: TranCurrent,
    },
    Tuple(TranVoltage, TranCurrent),
    Unit,
}

#[derive(Debug, Clone, FromSaved)]
#[allow(unused)]
pub struct NamedFields {
    vout: TranVoltage,
    iout: TranCurrent,
}

#[derive(Debug, Clone, FromSaved)]
#[allow(unused)]
pub struct NewType(NamedFields);

#[derive(Debug, Clone, FromSaved)]
#[allow(unused)]
pub struct Tuple(NamedFields, SavedEnum);
