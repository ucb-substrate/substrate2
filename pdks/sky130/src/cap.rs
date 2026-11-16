//! Capacitors.

use arcstr::ArcStr;
use rust_decimal::Decimal;
use scir::ParamValue;
use serde::{Deserialize, Serialize};
use substrate::{
    block::Block,
    schematic::{CellBuilder, PrimitiveBinding, Schematic},
    types::{TwoTerminalIo, schematic::IoNodeBundle},
};

use crate::{Primitive, Sky130};

/// A variety of MIM capacitor.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub enum MimCapKind {
    /// `sky130_fd_pr__cap_mim_m3_1`: a capm top plate over a met3 bottom plate, contacted from met4.
    Capm,
    /// `sky130_fd_pr__cap_mim_m3_2`: a cap2m top plate over a met4 bottom plate, contacted from met5.
    Cap2m,
}

impl MimCapKind {
    pub(crate) fn open_subckt(&self) -> ArcStr {
        match self {
            Self::Capm => arcstr::literal!("sky130_fd_pr__cap_mim_m3_1"),
            Self::Cap2m => arcstr::literal!("sky130_fd_pr__cap_mim_m3_2"),
        }
    }
}

/// A MIM capacitor with top plate `p` and bottom plate `n`.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize, Block)]
#[substrate(io = "TwoTerminalIo")]
pub struct MimCap {
    /// The capacitor variety.
    pub kind: MimCapKind,
    /// The width of each unit capacitor, in nm.
    pub w: i64,
    /// The length of each unit capacitor, in nm.
    pub l: i64,
    /// The number of unit capacitors in parallel.
    pub m: i64,
}

impl MimCap {
    /// SPICE instance parameters, with `w` and `l` in microns.
    pub(crate) fn params(&self) -> [(ArcStr, ParamValue); 3] {
        [
            (arcstr::literal!("w"), Decimal::new(self.w, 3).into()),
            (arcstr::literal!("l"), Decimal::new(self.l, 3).into()),
            (arcstr::literal!("m"), Decimal::from(self.m).into()),
        ]
    }
}

impl Schematic for MimCap {
    type Schema = Sky130;
    type NestedData = ();
    fn schematic(
        &self,
        io: &IoNodeBundle<Self>,
        cell: &mut CellBuilder<Sky130>,
    ) -> substrate::error::Result<Self::NestedData> {
        let mut prim = PrimitiveBinding::new(Primitive::MimCap(*self));
        prim.connect("P", io.p);
        prim.connect("N", io.n);
        cell.set_primitive(prim);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use scir::netlist::ConvertibleNetlister;
    use spice::Spice;
    use substrate::{context::Context, schematic::ConvertSchema};

    use super::*;
    use crate::Sky130OpenSchema;

    #[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize, Block)]
    #[substrate(io = "TwoTerminalIo")]
    struct MimCapWrapper(MimCap);

    impl Schematic for MimCapWrapper {
        type Schema = Sky130;
        type NestedData = ();
        fn schematic(
            &self,
            io: &IoNodeBundle<Self>,
            cell: &mut CellBuilder<Sky130>,
        ) -> substrate::error::Result<Self::NestedData> {
            cell.instantiate_connected(self.0, io);
            Ok(())
        }
    }

    fn open_netlist(cap: MimCap) -> String {
        let ctx = Context::builder().install(Sky130::open("")).build();
        let lib = ctx
            .export_scir(ConvertSchema::<_, Spice>::new(ConvertSchema::<
                _,
                Sky130OpenSchema,
            >::new(
                MimCapWrapper(cap)
            )))
            .unwrap();
        let mut buf = Vec::new();
        Spice
            .write_scir_netlist(&lib.scir, &mut buf, Default::default())
            .unwrap();
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn mim_cap_open_netlist() {
        for (kind, subckt) in [
            (MimCapKind::Capm, "sky130_fd_pr__cap_mim_m3_1"),
            (MimCapKind::Cap2m, "sky130_fd_pr__cap_mim_m3_2"),
        ] {
            let netlist = open_netlist(MimCap {
                kind,
                w: 2_000,
                l: 3_070,
                m: 4,
            });
            let line = format!(" p n {subckt} l=3.070 m=4 w=2.000\n");
            assert!(netlist.contains(&line), "missing `{line}` in:\n{netlist}");
        }
    }
}
