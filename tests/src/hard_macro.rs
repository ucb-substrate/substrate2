use crate::shared::buffer::BufferIo;
use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use sky130pdk::Sky130OpenPdk;
use spice::parser::conv::ScirConverter;
use std::collections::HashMap;
use std::sync::Arc;
use substrate::io::{Flatten, Node};
use substrate::schematic::{HasSchematic, HasSchematicImpl, PrimitiveDevice};
use substrate::Block;

#[derive(Block, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[substrate(
    io = "BufferIo",
    // schematic(path = "/path/to/schematic", fmt = "spectre")
)]
pub struct BufferHardMacro;

impl HasSchematic for BufferHardMacro {
    type Data = ();
}

impl HasSchematicImpl<Sky130OpenPdk> for BufferHardMacro {
    fn schematic(
        &self,
        io: &<<Self as substrate::block::Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::CellBuilder<Sky130OpenPdk, Self>,
    ) -> substrate::error::Result<Self::Data> {
        let path = crate::paths::test_data("spice/buffer.spice");
        let parsed = spice::parser::Parser::parse_file(path).unwrap();
        let mut conv = ScirConverter::new("buffer", &parsed.ast);
        conv.blackbox("sky130_fd_pr__nfet_01v8");
        conv.blackbox("sky130_fd_pr__pfet_01v8");
        let lib = Arc::new(conv.convert().unwrap());
        let cell_id = lib.cell_id_named("buffer");
        let connections: HashMap<ArcStr, Vec<Node>> = HashMap::from_iter([
            ("din".into(), io.din.flatten_vec()),
            ("dout".into(), io.dout.flatten_vec()),
            ("vdd".into(), io.vdd.flatten_vec()),
            ("vss".into(), io.vss.flatten_vec()),
        ]);
        cell.add_primitive(PrimitiveDevice::ScirInstance {
            lib,
            cell: cell_id,
            name: "buffer_hard_macro_inner".into(),
            connections,
        });
        Ok(())
    }
}

#[test]
fn export_hard_macro() {
    use crate::shared::pdk::sky130_open_ctx;

    let ctx = sky130_open_ctx();
    let lib = ctx.export_scir(BufferHardMacro);
    assert_eq!(lib.scir.cells().count(), 3);

    println!("SCIR Library:\n{:?}", lib.scir);

    let mut buf: Vec<u8> = Vec::new();
    let includes = Vec::new();
    let netlister = spectre::netlist::Netlister::new(&lib.scir, &includes, &mut buf);
    netlister.export().unwrap();
    let string = String::from_utf8(buf).unwrap();
    println!("Netlist:\n{}", string);
}
