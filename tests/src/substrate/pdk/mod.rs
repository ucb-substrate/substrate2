use std::collections::HashMap;

use scir::Expr;
use serde::{Deserialize, Serialize};
use substrate::block::Block;
use substrate::pdk::layers::{GdsLayerSpec, Layer};
use substrate::schematic::{HasSchematic, HasSchematicImpl, PrimitiveDevice};
use substrate::Io;
use substrate::{context::Context, pdk::Pdk};

use self::layers::{ExamplePdkALayers, ExamplePdkBLayers};

pub mod layers;

pub struct ExamplePdkA;

impl Pdk for ExamplePdkA {
    type Layers = ExamplePdkALayers;
}

pub struct ExamplePdkB;

impl Pdk for ExamplePdkB {
    type Layers = ExamplePdkBLayers;
}

/// A MOSFET in PDK A.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct MosA {
    pub w: i64,
    pub l: i64,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Io)]
pub struct MosIo {

}

impl Block for MosA {
    type Io = ();
    fn id() -> arcstr::ArcStr {
        arcstr::literal!("mos_a")
    }
    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("mos_a_w{}_l{}", self.w, self.l)
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl HasSchematic for MosA {
    type Data = ();
}

impl HasSchematicImpl<ExamplePdkA> for MosA {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::CellBuilder<ExamplePdkA, Self>,
    ) -> substrate::error::Result<Self::Data> {
        cell.add_primitive(PrimitiveDevice::RawInstance {
            ports: vec![],
            cell: arcstr::literal!("example_pdk_mos_a"),
            params: HashMap::from_iter([
                (arcstr::literal!("w"), Expr::NumericLiteral(self.w.into())),
                (arcstr::literal!("l"), Expr::NumericLiteral(self.l.into())),
            ]),
        });
        Ok(())
    }
}

#[test]
fn test_pdk_layers() {
    let ctx = Context::new(ExamplePdkA);

    assert_eq!(
        ctx.pdk.layers.met1a.as_ref(),
        ctx.pdk.layers.met1a_drawing.as_ref()
    );
    assert_eq!(ctx.pdk.layers.met1a.info().gds, Some(GdsLayerSpec(68, 20)));
    assert_eq!(
        ctx.pdk.layers.met1a_drawing.info().gds,
        Some(GdsLayerSpec(68, 20))
    );
    assert_eq!(
        ctx.pdk.layers.met1a_pin.info().gds,
        Some(GdsLayerSpec(68, 16))
    );
    assert_eq!(
        ctx.pdk.layers.met1a_label.info().gds,
        Some(GdsLayerSpec(68, 5))
    );
    assert_eq!(ctx.pdk.layers.met2a.info().gds, Some(GdsLayerSpec(69, 20)));

    assert_eq!(ctx.pdk.layers.polya.custom_property(), 5)
}
