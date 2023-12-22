use crate::paths::test_data;
use crate::shared::buffer::BufferIo;

use serde::{Deserialize, Serialize};
use sky130pdk::Sky130Pdk;
#[cfg(feature = "spectre")]
use spectre::Spectre;
#[cfg(feature = "spectre")]
use spice::netlist::NetlisterInstance;
use spice::Spice;
use substrate::block::Block;
use substrate::io::schematic::HardwareType;
use substrate::layout::Layout;
use substrate::schematic::netlist::ConvertibleNetlister;
use substrate::schematic::{CellBuilder, ExportsNestedData, Schematic};
use test_log::test;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, Block, Layout)]
#[substrate(io = "BufferIo")]
#[substrate(layout(
    source = "crate::paths::test_data(\"gds/buffer.gds\")",
    name = "buffer",
    fmt = "gds",
    pdk = "Sky130Pdk"
))]
pub struct BufferHardMacro;

impl ExportsNestedData for BufferHardMacro {
    type NestedData = ();
}

impl Schematic<Sky130Pdk> for BufferHardMacro {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<Sky130Pdk>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut scir = Spice::scir_cell_from_file(test_data("spice/buffer.spice"), "buffer")
            .convert_schema::<Sky130Pdk>()?;

        scir.connect("din", io.din);
        scir.connect("dout", io.dout);
        scir.connect("vss", io.vss);
        scir.connect("vdd", io.vdd);

        cell.set_scir(scir);
        Ok(())
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, Block)]
#[substrate(io = "BufferIo")]
pub struct BufferInlineHardMacro;

impl ExportsNestedData for BufferInlineHardMacro {
    type NestedData = ();
}

impl Schematic<Sky130Pdk> for BufferInlineHardMacro {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<Sky130Pdk>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut scir = Spice::scir_cell_from_str(
            r#"
                * CMOS buffer

                .subckt buffer din dout vdd vss
                X0 din dinb vdd vss inverter
                X1 dinb dout vdd vss inverter
                .ends

                .subckt inverter din dout vdd vss
                X0 dout din vss vss sky130_fd_pr__nfet_01v8 w=2 l=0.15
                X1 dout din vdd vdd sky130_fd_pr__pfet_01v8 w=4 l=0.15
                .ends
            "#,
            "buffer",
        )
        .convert_schema::<Sky130Pdk>()?;

        scir.connect("din", io.din);
        scir.connect("dout", io.dout);
        scir.connect("vss", io.vss);
        scir.connect("vdd", io.vdd);

        cell.set_scir(scir);
        Ok(())
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, Block)]
#[substrate(io = "crate::shared::vdivider::VdividerFlatIo")]
pub struct VdividerDuplicateSubckt;

impl ExportsNestedData for VdividerDuplicateSubckt {
    type NestedData = ();
}

impl Schematic<Spice> for VdividerDuplicateSubckt {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<Spice>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut scir = Spice::scir_cell_from_file(
            test_data("spice/vdivider_duplicate_subckt.spice"),
            "vdivider",
        );

        scir.connect("vdd", io.vdd);
        scir.connect("vss", io.vss);
        scir.connect("out", io.out);

        cell.set_scir(scir);
        Ok(())
    }
}

#[test]
fn export_hard_macro() {
    use crate::shared::pdk::sky130_open_ctx;

    let ctx = sky130_open_ctx();
    let lib = ctx.export_scir::<Sky130Pdk, _>(BufferHardMacro).unwrap();
    assert_eq!(lib.scir.cells().count(), 3);

    println!("SCIR Library:\n{:#?}", lib.scir);

    let spice_lib = lib.scir.convert_schema::<Spice>().unwrap().build().unwrap();

    let mut buf: Vec<u8> = Vec::new();
    Spice
        .write_scir_netlist(&spice_lib, &mut buf, Default::default())
        .unwrap();
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
    let netlister = NetlisterInstance::new(&Spectre {}, &spectre_lib, &mut buf, Default::default());
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
    Spice
        .write_scir_netlist(&spice_lib, &mut buf, Default::default())
        .unwrap();
    let string = String::from_utf8(buf).unwrap();
    println!("Netlist:\n{}", string);
}
