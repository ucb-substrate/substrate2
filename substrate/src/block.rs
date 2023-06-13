use arcstr::ArcStr;
use serde::{Deserialize, Serialize};

pub trait Block: Serialize + Deserialize<'static> {
    fn id() -> ArcStr;

    fn name(&self) -> ArcStr {
        arcstr::literal!("unnamed")
    }

    // TODO: Add metadata
}
