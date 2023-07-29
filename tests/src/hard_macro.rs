use crate::shared::buffer::BufferIo;

use serde::{Deserialize, Serialize};
use sky130pdk::Sky130OpenPdk;

use substrate::Block;
use substrate::{HasLayoutImpl, HasSchematicImpl};
use test_log::test;

#[derive(
    Clone,
    Debug,
    Hash,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
    Block,
    HasSchematicImpl,
    HasLayoutImpl,
)]
#[substrate(io = "BufferIo")]
#[substrate(schematic(
    source = "crate::paths::test_data(\"spice/buffer.spice\")",
    name = "buffer",
    fmt = "spice",
    pdk = "Sky130OpenPdk"
))]
#[cfg_attr(
    feature = "spectre",
    substrate(schematic(
        source = "crate::paths::test_data(\"spice/buffer_commercial.spice\")",
        name = "buffer",
        fmt = "spice",
        pdk = "sky130pdk::Sky130CommercialPdk"
    ))
)]
#[substrate(layout(
    source = "crate::paths::test_data(\"gds/buffer.gds\")",
    name = "buffer",
    fmt = "gds",
    pdk = "Sky130OpenPdk"
))]
pub struct BufferHardMacro;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, Block, HasSchematicImpl)]
#[substrate(io = "BufferIo")]
#[substrate(schematic(
    source = "r#\"
        * CMOS buffer

        .subckt buffer din dout vdd vss
        X0 din dinb vdd vss inverter
        X1 dinb dout vdd vss inverter
        .ends

        .subckt inverter din dout vdd vss
        X0 dout din vss vss sky130_fd_pr__nfet_01v8 w=2 l=0.15
        X1 dout din vdd vdd sky130_fd_pr__pfet_01v8 w=4 l=0.15
        .ends
    \"#",
    name = "buffer",
    fmt = "inline-spice",
    pdk = "Sky130OpenPdk"
))]
pub struct BufferInlineHardMacro;

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

#[test]
fn export_hard_macro_gds() {
    use crate::shared::pdk::sky130_open_ctx;

    let ctx = sky130_open_ctx();
    ctx.write_layout(
        BufferHardMacro,
        crate::paths::get_path("export_hard_macro_gds", "layout.gds"),
    )
    .unwrap();
}

#[test]
#[cfg(feature = "spectre")]
fn export_hard_macro_in_another_pdk() {
    use crate::shared::pdk::sky130_commercial_ctx;

    let ctx = sky130_commercial_ctx();
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

#[test]
fn export_inline_hard_macro() {
    use crate::shared::pdk::sky130_open_ctx;

    let ctx = sky130_open_ctx();
    let lib = ctx.export_scir(BufferInlineHardMacro);
    assert_eq!(lib.scir.cells().count(), 3);

    println!("SCIR Library:\n{:?}", lib.scir);

    let mut buf: Vec<u8> = Vec::new();
    let includes = Vec::new();
    let netlister = spectre::netlist::Netlister::new(&lib.scir, &includes, &mut buf);
    netlister.export().unwrap();
    let string = String::from_utf8(buf).unwrap();
    println!("Netlist:\n{}", string);
}
