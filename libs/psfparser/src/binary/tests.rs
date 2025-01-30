use approx::assert_relative_eq;

use crate::analysis::ac::AcData;
use crate::analysis::dc::DcData;
use crate::analysis::transient::TransientData;

use super::*;

pub(crate) static TRAN_EXAMPLE_PSFBIN_1: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/tranbin1.tran.tran"
));

pub(crate) static VDIV_SIN_PSFBIN: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/vdiv_sin_bin.tran.tran"
));

pub(crate) static SRAM_TINY_PSFBIN: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/sram_tiny_bin.tran.tran"
));

pub(crate) static AC_ZOUT_PSFBIN: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/AcZout.ac"));

pub(crate) static DC_OP_PSFBIN: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/dcop.bin.dc"));

pub(crate) static DC_SWEEPI_PSFBIN: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/dcsweepi.bin.dc"
));

pub(crate) static DC_SWEEPV_PSFBIN: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/dcsweepv.bin.dc"
));

#[test]
fn test_header() {
    let toc = parse_toc(TRAN_EXAMPLE_PSFBIN_1);
    let header = parse_header(TRAN_EXAMPLE_PSFBIN_1, &toc.data[&SectionKind::Header]);
    println!("Header: {:?}", header);
}

#[test]
fn test_types() {
    let toc = parse_toc(TRAN_EXAMPLE_PSFBIN_1);
    let types = parse_types(TRAN_EXAMPLE_PSFBIN_1, &toc.data[&SectionKind::Type]);
    println!("Types: {:?}", types);
}

#[test]
fn test_traces() {
    let toc = parse_toc(TRAN_EXAMPLE_PSFBIN_1);
    println!("ToC: {:?}", toc);
    let traces = parse_traces(TRAN_EXAMPLE_PSFBIN_1, &toc.data[&SectionKind::Trace]);
    let expected = vec![Trace::Group(TraceGroup {
        name: "group",
        count: 3,
        id: GroupId(24),
        signals: vec![
            SignalRef {
                id: TraceId(25),
                name: "out",
                type_id: TypeId(11),
                properties: Properties { values: vec![] },
            },
            SignalRef {
                id: TraceId(26),
                name: "vdd",
                type_id: TypeId(11),
                properties: Properties { values: vec![] },
            },
            SignalRef {
                id: TraceId(27),
                name: "Vvdd:p",
                type_id: TypeId(12),
                properties: Properties { values: vec![] },
            },
        ],
    })];
    println!("Traces: {:?}", traces);
    assert_eq!(traces, expected);
}

#[test]
fn test_values() {
    let toc = parse_toc(TRAN_EXAMPLE_PSFBIN_1);
    println!("ToC: {:?}", toc);
    let mut parser = PsfParser::new(TRAN_EXAMPLE_PSFBIN_1);
    parser.parse();
}

#[test]
fn test_sweeps() {
    let toc = parse_toc(TRAN_EXAMPLE_PSFBIN_1);
    println!("ToC: {:?}", toc);
    let sweeps = parse_sweeps(TRAN_EXAMPLE_PSFBIN_1, &toc.data[&SectionKind::Sweep]);
    println!("Sweeps: {:?}", sweeps);
}

#[test]
fn test_to_transient_1() {
    use crate::binary::parse;
    let ast = parse(TRAN_EXAMPLE_PSFBIN_1).unwrap();
    println!("ast = {ast:#?}");
    let data = TransientData::from_binary(ast);
    assert_eq!(data.signals.len(), 4);
    assert_eq!(
        data.signal("time")
            .expect("should contain a time signal")
            .len(),
        11
    );
}

#[test]
fn parses_vdiv_sin_bin() {
    use crate::binary::parse;
    let ast = parse(VDIV_SIN_PSFBIN).unwrap();
    let data = TransientData::from_binary(ast);
    assert_eq!(data.signals.len(), 4);
    assert_eq!(
        data.signal("time")
            .expect("should contain a time signal")
            .len(),
        16001
    );
}

#[test]
fn parses_sram_tiny_bin() {
    use crate::binary::parse;
    let ast = parse(SRAM_TINY_PSFBIN).unwrap();
    let data = TransientData::from_binary(ast);
    assert_eq!(data.signals.len(), 1321);
    assert_eq!(
        data.signal("time")
            .expect("should contain a time signal")
            .len(),
        201
    );
}

#[test]
fn parses_ac() {
    let ast = parse(AC_ZOUT_PSFBIN).expect("Failed to parse ac PSF file");
    let data = AcData::from_binary(ast);
    assert_eq!(data.signals.len(), 2);
    assert_eq!(data.freq.len(), 78);
}

#[test]
fn parses_dc_op() {
    let ast = parse(DC_OP_PSFBIN).expect("Failed to parse dc op PSF file");
    let data = DcData::from_binary(ast);
    match data {
        DcData::Op(op) => {
            assert_eq!(op.signals.len(), 3);
            assert_relative_eq!(op.signals["vin"], 1., max_relative = 1e-9);
            assert_relative_eq!(
                op.signals["mid"],
                0.2 * 1. - 1e-3 * 800.,
                max_relative = 1e-9
            );
        }
        _ => panic!("should be op"),
    };
}

#[test]
fn parses_dc_sweep() {
    let ast = parse(DC_SWEEPI_PSFBIN).expect("Failed to parse dc PSF file with swept current");
    let data = DcData::from_binary(ast);
    match data {
        DcData::Sweep(swp) => {
            assert_eq!(swp.sweep_var, "ILOAD");
            let mid = &swp.signals["mid"];
            assert_eq!(mid.len(), 102);
            assert_eq!(mid.len(), swp.sweep_values.len());
            for (iload, mid) in swp.sweep_values.iter().copied().zip(mid.iter().copied()) {
                assert_relative_eq!(mid, 0.2 * 1. - iload * 800., max_relative = 1e-9);
            }
        }
        _ => panic!("should be sweep"),
    };
    let ast = parse(DC_SWEEPV_PSFBIN).expect("Failed to parse dc PSF file with swept voltage");
    let data = DcData::from_binary(ast);
    match data {
        DcData::Sweep(swp) => {
            assert_eq!(swp.sweep_var, "VDD");
            let mid = &swp.signals["mid"];
            assert_eq!(mid.len(), 102);
            assert_eq!(mid.len(), swp.sweep_values.len());
            for (vdd, mid) in swp.sweep_values.iter().copied().zip(mid.iter().copied()) {
                assert_relative_eq!(mid, 0.2 * vdd - 1e-3 * 800., max_relative = 1e-9);
            }
        }
        _ => panic!("should be sweep"),
    };
}
