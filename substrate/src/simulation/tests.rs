use substrate::simulation::data::{tran, FromSaved};

#[derive(Debug, Clone, FromSaved)]
#[allow(unused)]
pub enum SavedEnum {
    Fields {
        vout: tran::Voltage,
        iout: tran::Current,
    },
    Tuple(tran::Voltage, tran::Current),
    Unit,
}

#[derive(Debug, Clone, FromSaved)]
#[allow(unused)]
pub struct NamedFields {
    vout: tran::Voltage,
    iout: tran::Current,
}

#[derive(Debug, Clone, FromSaved)]
#[allow(unused)]
pub struct NewType(NamedFields);

#[derive(Debug, Clone, FromSaved)]
#[allow(unused)]
pub struct Tuple(NamedFields, SavedEnum);
