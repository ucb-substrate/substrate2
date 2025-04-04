//! Resistors.

use geometry::rect::Rect;
use layir::Shape;
use serde::{Deserialize, Serialize};
use substrate::{
    block::Block,
    layout::Layout,
    types::{layout::PortGeometry, TwoTerminalIo, TwoTerminalIoView},
};

use crate::{layers::Sky130Layer, Sky130};

const SLOTTED_LICON_W: i64 = 190;
const SLOTTED_LICON_L: i64 = 2_000;
const SLOTTED_LICON_SPACING: i64 = 510;

/// A precision p+ poly resistor.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize, Block)]
#[substrate(io = "TwoTerminalIo")]
pub struct PrecisionResistor {
    /// The width of the resistor.
    pub width: PrecisionResistorWidth,
    /// The length of the resistor.
    ///
    /// Resistance is roughly proportional to the resistor's length.
    pub length: i64,
}

/// The allowed widths of [`PrecisionResistor`]s in SKY130.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize, Ord, PartialOrd)]
pub enum PrecisionResistorWidth {
    /// 0.35um width.
    W035,
    /// 0.69um width.
    W069,
    /// 1.41um width.
    W141,
    /// 2.85um width.
    W285,
    /// 5.73um width.
    W573,
}

impl PrecisionResistorWidth {
    /// The width in layout database units.
    pub fn dbu(&self) -> i64 {
        match *self {
            Self::W035 => 350,
            Self::W069 => 690,
            Self::W141 => 1_410,
            Self::W285 => 2_850,
            Self::W573 => 5_730,
        }
    }

    /// The number of licons allowed within a single terminal.
    fn num_licons(&self) -> usize {
        match *self {
            Self::W035 => 1,
            Self::W069 => 1,
            Self::W141 => 2,
            Self::W285 => 4,
            Self::W573 => 8,
        }
    }
}

impl Layout for PrecisionResistor {
    type Schema = Sky130;
    type Data = ();
    type Bundle = TwoTerminalIoView<substrate::types::codegen::PortGeometryBundle<Sky130>>;
    fn layout(
        &self,
        cell: &mut substrate::layout::CellBuilder<Self::Schema>,
    ) -> substrate::error::Result<(Self::Bundle, Self::Data)> {
        let poly_w = 2 * (80 + SLOTTED_LICON_L) + self.length;
        let poly_h = self.width.dbu();
        let poly = Rect::from_sides(0, 0, poly_w, poly_h);
        cell.draw(Shape::new(Sky130Layer::Poly, poly))?;

        let n_licon = self.width.num_licons() as i64;
        assert!(n_licon > 0);
        let licon_y0 =
            (poly_h - (SLOTTED_LICON_W * n_licon + SLOTTED_LICON_SPACING * (n_licon - 1))) / 2;
        let licon_y1 = poly_h - licon_y0;
        let poly_res_x0 = 80 + SLOTTED_LICON_L;
        let poly_res_x1 = poly_w - 80 - SLOTTED_LICON_L;
        for i in 0..n_licon {
            let y0 = licon_y0 + i * (SLOTTED_LICON_W + SLOTTED_LICON_SPACING);
            let licon_left = Rect::from_sides(80, y0, poly_res_x0, y0 + SLOTTED_LICON_W);
            cell.draw(Shape::new(Sky130Layer::Licon1, licon_left))?;
            let licon_right = Rect::from_sides(poly_res_x1, y0, poly_w - 80, y0 + SLOTTED_LICON_W);
            cell.draw(Shape::new(Sky130Layer::Licon1, licon_right))?;
        }

        let li1_left = Rect::from_sides(0, licon_y0 - 80, 80 + SLOTTED_LICON_L + 80, licon_y1 + 80);
        let li1_left = Shape::new(Sky130Layer::Li1, li1_left);
        cell.draw(li1_left.clone())?;
        let li1_right = Rect::from_sides(poly_res_x1 - 80, licon_y0 - 80, poly_w, licon_y1 + 80);
        let li1_right = Shape::new(Sky130Layer::Li1, li1_right);
        cell.draw(li1_right.clone())?;

        cell.draw(Shape::new(Sky130Layer::Npc, poly.expand_all(95)))?;
        cell.draw(Shape::new(Sky130Layer::Psdm, poly.expand_all(110)))?;
        cell.draw(Shape::new(Sky130Layer::Urpm, poly.expand_all(200)))?;
        cell.draw(Shape::new(
            Sky130Layer::PolyRes,
            Rect::from_sides(poly_res_x0, 0, poly_res_x1, poly_h),
        ))?;

        let cx = poly_w / 2;
        cell.draw(Shape::new(
            Sky130Layer::PolyCut,
            Rect::from_sides(cx, 0, cx + 5, poly_h),
        ))?;

        Ok((
            TwoTerminalIoView {
                p: PortGeometry::new(li1_left),
                n: PortGeometry::new(li1_right),
            },
            (),
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::{layout::to_gds, tests::sky130_cds_ctx};
    use pegasus::{
        drc::{run_drc, DrcParams},
        RuleCheck,
    };
    use substrate::block::Block;

    use crate::{SKY130_DRC, SKY130_DRC_RULES_PATH, SKY130_LVS, SKY130_LVS_RULES_PATH};

    fn test_check_filter(check: &RuleCheck) -> bool {
        !["licon.12", "hvnwell.8"].contains(&check.name.as_ref())
    }

    #[test]
    #[ignore = "Cadence PDK does not yet support resistors"]
    fn precision_resistor_lvs_cds() {
        let work_dir = PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/build/precision_resistor_lvs_cds"
        ));
        let layout_path = work_dir.join("layout.gds");
        let ctx = sky130_cds_ctx();

        let dut = PrecisionResistor {
            width: PrecisionResistorWidth::W285,
            length: 6_000,
        };

        ctx.write_layout(dut, to_gds, &layout_path).unwrap();

        // Run DRC.
        let drc_dir = work_dir.join("drc");
        let data = run_drc(&DrcParams {
            work_dir: &drc_dir,
            layout_path: &layout_path,
            cell_name: &dut.name(),
            rules_dir: &PathBuf::from(SKY130_DRC),
            rules_path: &PathBuf::from(SKY130_DRC_RULES_PATH),
        })
        .expect("failed to run drc");

        assert_eq!(
            data.rule_checks
                .into_iter()
                .filter(test_check_filter)
                .count(),
            0,
            "layout was not DRC clean"
        );

        // // Run LVS.
        // let lvs_dir = work_dir.join("lvs");
        // let source_path = work_dir.join("schematic.spice");
        // let rawlib = ctx
        //     .export_scir(ConvertSchema::<_, Spice>::new(ConvertSchema::<
        //         _,
        //         Sky130CdsSchema,
        //     >::new(dut)))
        //     .unwrap();

        // Spice
        //     .write_scir_netlist_to_file(&rawlib.scir, &source_path, NetlistOptions::default())
        //     .expect("failed to write netlist");
        // let output = pegasus::lvs::run_lvs(&LvsParams {
        //     work_dir: &lvs_dir,
        //     layout_path: &layout_path,
        //     layout_cell_name: &dut.name(),
        //     source_paths: &[source_path],
        //     source_cell_name: &dut.name(),
        //     rules_dir: &PathBuf::from(SKY130_LVS),
        //     rules_path: &PathBuf::from(SKY130_LVS_RULES_PATH),
        // })
        // .expect("failed to run lvs");

        // assert_eq!(
        //     output.status,
        //     LvsStatus::Correct,
        //     "layout does not match netlist"
        // );
    }
}
