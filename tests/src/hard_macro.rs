use crate::shared::buffer::BufferIo;

use serde::{Deserialize, Serialize};
use sky130pdk::Sky130Pdk;

use scir::netlist::{NetlistKind, NetlisterInstance};
#[cfg(feature = "spectre")]
use spectre::Spectre;
use spice::Spice;
use substrate::block::Block;
use substrate::{layout::Layout, schematic::Schematic};
use test_log::test;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, Block, Schematic, Layout)]
#[substrate(io = "BufferIo", kind = "Scir", flatten)]
#[substrate(schematic(
    source = "crate::paths::test_data(\"spice/buffer.spice\")",
    name = "buffer",
    fmt = "spice",
    pdk = "Sky130Pdk"
))]
#[substrate(layout(
    source = "crate::paths::test_data(\"gds/buffer.gds\")",
    name = "buffer",
    fmt = "gds",
    pdk = "Sky130Pdk"
))]
pub struct BufferHardMacro;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, Block, Schematic)]
#[substrate(io = "BufferIo", kind = "Scir")]
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
    pdk = "Sky130Pdk"
))]
pub struct BufferInlineHardMacro;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, Block, Schematic)]
#[substrate(io = "crate::shared::vdivider::VdividerFlatIo", kind = "Scir", flatten)]
#[substrate(schematic(
    source = "crate::paths::test_data(\"spice/vdivider_duplicate_subckt.spice\")",
    name = "vdivider",
    fmt = "spice",
    pdk = "Sky130Pdk"
))]
pub struct VdividerDuplicateSubckt;

#[test]
fn export_hard_macro() {
    use crate::shared::pdk::sky130_open_ctx;

    let ctx = sky130_open_ctx();
    let lib = ctx.export_scir::<Sky130Pdk, _>(BufferHardMacro).unwrap();
    assert_eq!(lib.scir.cells().count(), 3);

    println!("SCIR Library:\n{:#?}", lib.scir);

    let spice_lib = lib.scir.convert_schema::<Spice>().unwrap().build().unwrap();

    let mut buf: Vec<u8> = Vec::new();
    let netlister = NetlisterInstance::new(NetlistKind::Cells, &Spice, &spice_lib, &[], &mut buf);
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
fn export_hard_macro_to_spectre() {
    use crate::shared::pdk::sky130_commercial_ctx;

    let ctx = sky130_commercial_ctx();
    let lib = ctx.export_scir::<Sky130Pdk, _>(BufferHardMacro).unwrap();
    assert_eq!(lib.scir.cells().count(), 3);

    println!("SCIR Library:\n{:?}", lib.scir);

    let spectre_lib = lib
        .scir
        .convert_schema::<Spectre>()
        .unwrap()
        .build()
        .unwrap();

    let mut buf: Vec<u8> = Vec::new();
    let netlister =
        NetlisterInstance::new(NetlistKind::Cells, &Spectre {}, &spectre_lib, &[], &mut buf);
    netlister.export().unwrap();
    let string = String::from_utf8(buf).unwrap();
    println!("Netlist:\n{}", string);
}

#[test]
fn export_inline_hard_macro() {
    use crate::shared::pdk::sky130_open_ctx;

    let ctx = sky130_open_ctx();
    let lib = ctx
        .export_scir::<Sky130Pdk, _>(BufferInlineHardMacro)
        .unwrap();
    assert_eq!(lib.scir.cells().count(), 2);

    println!("SCIR Library:\n{:?}", lib.scir);

    let spice_lib = lib.scir.convert_schema::<Spice>().unwrap().build().unwrap();

    let mut buf: Vec<u8> = Vec::new();
    let netlister = NetlisterInstance::new(NetlistKind::Cells, &Spice, &spice_lib, &[], &mut buf);
    netlister.export().unwrap();
    let string = String::from_utf8(buf).unwrap();
    println!("Netlist:\n{}", string);
}
