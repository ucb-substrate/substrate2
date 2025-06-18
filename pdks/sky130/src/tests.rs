use crate::corner::Sky130Corner;
use crate::layers::Sky130Layer;
use crate::layout::{to_gds, GDS_UNITS};
use crate::mos::{MosKind, MosLength, NmosTile, PmosTile};
use crate::stdcells::{And2, And2Io};
use crate::{convert_spice_mos, Primitive, Sky130, Sky130OpenSchema, Sky130SrcNdaSchema};
use approx::assert_abs_diff_eq;
use derive_where::derive_where;
use gds::GdsLibrary;
use gdsconv::conv::from_gds;
use gdsconv::import::GdsImportOpts;
use layir::Element;
use ngspice::Ngspice;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use scir::ParamValue;
use spectre::Spectre;
use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::PathBuf;
use substrate::block::Block;
use substrate::context::Context;
use substrate::schematic::schema::Schema;
use substrate::schematic::{CellBuilder, ConvertSchema, Schematic};
use substrate::simulation::waveform::TimeWaveform;
use substrate::types::schematic::Terminal;
use substrate::types::{TestbenchIo, TwoTerminalIo};
use unicase::UniCase;

const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");

#[inline]
pub(crate) fn get_path(test_name: &str, file_name: &str) -> PathBuf {
    PathBuf::from(BUILD_DIR).join(test_name).join(file_name)
}

/// Create a new Substrate context for the SKY130 open-source PDK.
///
/// Sets the PDK root to the value of the `SKY130_OPEN_PDK_ROOT`
/// environment variable.
///
/// # Panics
///
/// Panics if the `SKY130_OPEN_PDK_ROOT` environment variable is not set,
/// or if the value of that variable is not a valid UTF-8 string.
pub fn sky130_open_ctx() -> Context {
    let pdk_root = std::env::var("SKY130_OPEN_PDK_ROOT")
        .expect("the SKY130_OPEN_PDK_ROOT environment variable must be set");
    Context::builder()
        .install(Ngspice::default())
        .install(Sky130::open(pdk_root))
        .build()
}

/// Create a new Substrate context for the SKY130 SRC NDA PDK.
///
/// Sets the PDK root to the value of the `SKY130_SRC_NDA_PDK_ROOT`
/// environment variable and installs Spectre with default configuration.
///
/// # Panics
///
/// Panics if either the `SKY130_SRC_NDA_PDK_ROOT` or `SKY130_OPEN_PDK_ROOT` environment variable is not set,
/// or if the value of the variables are not valid UTF-8 strings.
pub fn sky130_src_nda_ctx() -> Context {
    // Open PDK needed for standard cells.
    let open_pdk_root = std::env::var("SKY130_OPEN_PDK_ROOT")
        .expect("the SKY130_OPEN_PDK_ROOT environment variable must be set");
    let src_nda_pdk_root = std::env::var("SKY130_SRC_NDA_PDK_ROOT")
        .expect("the SKY130_SRC_NDA_PDK_ROOT environment variable must be set");
    Context::builder()
        .install(Spectre::default())
        .install(Sky130::src_nda(open_pdk_root, src_nda_pdk_root))
        .build()
}

/// Create a new Substrate context for the SKY130 CDS PDK.
///
/// # Panics
///
/// Panics if the `SKY130_CDS_PDK_ROOT` environment variable is not set,
/// or if its value is not a valid UTF-8 string.
pub fn sky130_cds_ctx() -> Context {
    let pdk_root = std::env::var("SKY130_CDS_PDK_ROOT")
        .expect("the SKY130_CDS_PDK_ROOT environment variable must be set");
    Context::builder()
        .install(spectre::Spectre::default())
        .install(Sky130::cds_only(pdk_root))
        .build()
}

#[derive_where(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct And2Tb<S> {
    schema: PhantomData<fn() -> S>,
    vdd: Decimal,
    a: Decimal,
    b: Decimal,
}

impl<S: Any> Block for And2Tb<S> {
    type Io = TestbenchIo;

    fn name(&self) -> arcstr::ArcStr {
        arcstr::literal!("and2_tb")
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

pub trait SupportsAnd2Tb: Schema {
    type And2: Block<Io = And2Io> + Schematic<Schema = Self>;
    type DcVsource: Block<Io = TwoTerminalIo> + Schematic<Schema = Self>;

    fn and2(params: And2) -> Self::And2;
    fn dc_vsource(v: Decimal) -> Self::DcVsource;
}

impl SupportsAnd2Tb for Ngspice {
    type And2 = ConvertSchema<ConvertSchema<And2, Sky130OpenSchema>, Ngspice>;
    type DcVsource = ngspice::blocks::Vsource;

    fn and2(params: And2) -> Self::And2 {
        ConvertSchema::new(ConvertSchema::new(params))
    }
    fn dc_vsource(v: Decimal) -> Self::DcVsource {
        ngspice::blocks::Vsource::dc(v)
    }
}

impl SupportsAnd2Tb for Spectre {
    type And2 = ConvertSchema<ConvertSchema<And2, Sky130SrcNdaSchema>, Spectre>;
    type DcVsource = spectre::blocks::Vsource;

    fn and2(params: And2) -> Self::And2 {
        ConvertSchema::new(ConvertSchema::new(params))
    }
    fn dc_vsource(v: Decimal) -> Self::DcVsource {
        spectre::blocks::Vsource::dc(v)
    }
}

impl<S: SupportsAnd2Tb> Schematic for And2Tb<S> {
    type Schema = S;
    type NestedData = Terminal;
    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        let vddsrc = cell.instantiate(S::dc_vsource(self.vdd));
        let asrc = cell.instantiate(S::dc_vsource(self.a));
        let bsrc = cell.instantiate(S::dc_vsource(self.b));
        let and2 = cell.instantiate_blocking(S::and2(And2::S0)).unwrap();

        cell.connect(io.vss, vddsrc.io().n);
        cell.connect_multiple(&[
            vddsrc.io().n,
            asrc.io().n,
            bsrc.io().n,
            and2.io().pwr.vgnd,
            and2.io().pwr.vnb,
        ]);
        cell.connect_multiple(&[vddsrc.io().p, and2.io().pwr.vpwr, and2.io().pwr.vpb]);
        cell.connect(and2.io().a, asrc.io().p);
        cell.connect(and2.io().b, bsrc.io().p);

        Ok(and2.io().x)
    }
}

#[test]
fn sky130_and2_ngspice() {
    let test_name = "sky130_and2_ngspice";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_open_ctx();

    for (a, b, expected) in [(dec!(1.8), dec!(1.8), 1.8f64), (dec!(1.8), dec!(0), 0f64)] {
        let sim = ctx
            .get_sim_controller(
                And2Tb {
                    schema: PhantomData,
                    vdd: dec!(1.8),
                    a,
                    b,
                },
                &sim_dir,
            )
            .expect("failed to create sim controller");
        let mut opts = ngspice::Options::default();
        sim.set_option(Sky130Corner::Tt, &mut opts);
        let vout = sim
            .simulate(
                opts,
                ngspice::tran::Tran {
                    step: dec!(1e-9),
                    stop: dec!(2e-9),
                    ..Default::default()
                },
            )
            .expect("failed to run simulation");
        assert_abs_diff_eq!(vout.v.last_x().unwrap(), expected, epsilon = 1e-6);
    }
}

#[cfg(feature = "spectre")]
#[test]
fn sky130_and2_monte_carlo_spectre() {
    let test_name = "sky130_and2_spectre";
    let sim_dir = get_path(test_name, "sim/");
    let ctx = sky130_src_nda_ctx();

    for (a, b, expected) in [
        (dec!(1.8), dec!(1.8), 1.8f64),
        (dec!(1.8), dec!(0), 0f64),
        (dec!(0), dec!(1.8), 0f64),
        (dec!(0), dec!(0), 0f64),
    ] {
        let sim = ctx
            .get_sim_controller(
                And2Tb {
                    schema: PhantomData,
                    vdd: dec!(1.8),
                    a,
                    b,
                },
                &sim_dir,
            )
            .expect("failed to create sim controller");
        let mut opts = spectre::Options::default();
        sim.set_option(Sky130Corner::Tt, &mut opts);
        let mc_vout = sim
            .simulate(
                opts,
                spectre::analysis::montecarlo::MonteCarlo {
                    variations: spectre::analysis::montecarlo::Variations::All,
                    numruns: 4,
                    seed: None,
                    firstrun: None,
                    analysis: spectre::analysis::tran::Tran {
                        stop: dec!(2e-9),
                        errpreset: Some(spectre::ErrPreset::Conservative),
                        ..Default::default()
                    },
                },
            )
            .expect("failed to run simulation");
        assert_eq!(
            mc_vout.len(),
            4,
            "MonteCarlo output did not contain data from the correct number of runs"
        );
        for vout in &*mc_vout {
            assert_abs_diff_eq!(vout.v.last_x().unwrap(), expected, epsilon = 1e-6);
        }
    }
}

#[test]
fn nfet_01v8_layout() {
    let test_name = "nfet_01v8_layout";
    let ctx = sky130_src_nda_ctx();
    let layout_path = get_path(test_name, "layout.gds");

    ctx.write_layout(
        NmosTile::new(1_600, MosLength::L150, 4),
        to_gds,
        layout_path,
    )
    .unwrap();
}

#[test]
fn stdcell_and2_layout() {
    let test_name = "stdcell_and2_layout";
    let ctx = sky130_src_nda_ctx();
    let layout_path = get_path(test_name, "layout.gds");

    ctx.write_layout(And2::S4, to_gds, layout_path).unwrap();
}

#[test]
fn import_gds() {
    let test_name = "import_gds";
    let ctx = sky130_src_nda_ctx();
    let layout_path = get_path(test_name, "layout.gds");

    ctx.write_layout(
        NmosTile::new(1_600, MosLength::L150, 4),
        to_gds,
        &layout_path,
    )
    .unwrap();

    let rawlib = GdsLibrary::load(layout_path).unwrap();
    let lib = gdsconv::import::import_gds(
        &rawlib,
        GdsImportOpts {
            units: Some(GDS_UNITS),
        },
    )
    .expect("failed to import to LayIR");
    let lib =
        from_gds::<Sky130Layer>(&lib).expect("failed to convert GDS library to sky130 library");
    let cell = lib
        .try_cell_named("nmos_tile_w1600_l150_nf4")
        .expect("no nmos tile found");
    for port in ["sd_0", "sd_1", "sd_2", "sd_3", "sd_4", "g_0", "g_1", "b"] {
        let port = cell.port(port);
        // Check that the port contains at least one shape and one text.
        assert!(port.elements().any(|e| matches!(e, Element::Shape(_))));
        assert!(port.elements().any(|e| matches!(e, Element::Text(_))));
    }
}

#[test]
fn pfet_01v8_layout() {
    let test_name = "pfet_01v8_layout";
    let ctx = sky130_src_nda_ctx();
    let layout_path = get_path(test_name, "layout.gds");

    ctx.write_layout(
        PmosTile::new(1_600, MosLength::L150, 4),
        to_gds,
        layout_path,
    )
    .unwrap();
}

#[test]
fn test_convert_spice_mos() {
    let params = HashMap::from_iter([
        (
            UniCase::new(arcstr::literal!("w")),
            ParamValue::Numeric(dec!(2)),
        ),
        (
            UniCase::new(arcstr::literal!("l")),
            ParamValue::Numeric(dec!(0.15)),
        ),
        (
            UniCase::new(arcstr::literal!("nf")),
            ParamValue::Numeric(dec!(4)),
        ),
        (
            UniCase::new(arcstr::literal!("mult")),
            ParamValue::Numeric(dec!(2)),
        ),
        (
            UniCase::new(arcstr::literal!("m")),
            ParamValue::Numeric(dec!(3)),
        ),
    ]);
    let kind = "nshort";
    let prim = convert_spice_mos(kind, &params).expect("failed to convert mos");
    match prim {
        Primitive::Mos { kind, params } => {
            assert_eq!(kind, MosKind::Nfet01v8);
            assert_eq!(params.nf, 24);
            assert_eq!(params.w, 2_000);
            assert_eq!(params.l, 150);
        }
        _ => panic!("bad primitive"),
    }
}
