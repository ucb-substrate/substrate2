pub use scir::schema::{NoSchema, NoSchemaError};
use serde::{Deserialize, Serialize};
use substrate::pdk::layers::Layers;
use substrate::pdk::Pdk;

pub struct EmptyPdk;

#[derive(Layers)]
pub struct NoLayers;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct NoCorner;

impl Pdk for EmptyPdk {
    type Layers = NoLayers;
    type Corner = NoCorner;
    const LAYOUT_DB_UNITS: Option<rust_decimal::Decimal> = None;
}
