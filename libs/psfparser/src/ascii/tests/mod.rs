use crate::analysis::ac::AcData;
use crate::analysis::dc::DcData;
use crate::analysis::transient::TransientData;
use crate::ascii::ast::*;
use crate::ascii::frontend::parse;

#[test]
fn basic() {
    let input = r#"
    HEADER
    "PSFversion" "1.00"
    "integer value" 4
    "start" 0.0000
    "stop" 8.000e-08
    TYPE
    SWEEP
    TRACE
    " 1" GROUP 1
    "v(dout[0])" "V"
    VALUE
    END
    "#;

    let psf = parse(input).unwrap();
    assert_eq!(
        psf,
        PsfAst {
            header: Header {
                values: vec![
                    NamedValue {
                        name: "PSFversion",
                        value: Value::Str("1.00"),
                    },
                    NamedValue {
                        name: "integer value",
                        value: Value::Int(4),
                    },
                    NamedValue {
                        name: "start",
                        value: Value::Real(0f64),
                    },
                    NamedValue {
                        name: "stop",
                        value: Value::Real(8.0e-8),
                    },
                ]
            },
            types: Vec::new(),
            sweeps: Vec::new(),
            traces: vec![
                Trace::Group {
                    name: " 1",
                    count: 1
                },
                Trace::Signal {
                    name: "v(dout[0])",
                    units: "V"
                }
            ],
            values: Vec::new(),
        }
    )
}

pub(crate) static TRAN_EXAMPLE1_PSF: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/timeSweep1.tran.tran"
));

pub(crate) static TRAN_EXAMPLE2_PSF: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/timeSweep2.tran.tran"
));

pub(crate) static VDIV_SIN_PSF: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/vdiv_sin_ascii.tran.tran"
));

pub(crate) static SRAM_TINY_PSF: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/sram_tiny_ascii.tran.tran"
));

pub(crate) static AC_EXAMPLE_PSF: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/frequencySweep.ac"
));

static DC_EXAMPLE1_PSF: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/dc1.dc"));

static DC_EXAMPLE2_PSF: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/dc2.dc"));

#[test]
fn parses_transient_1() {
    let ast = parse(TRAN_EXAMPLE1_PSF).expect("Failed to parse transient PSF file");
    let data = TransientData::from_ascii(&ast);
    assert_eq!(data.signals.len(), 17);
}

#[test]
fn parses_transient_2() {
    let ast = parse(TRAN_EXAMPLE2_PSF).expect("Failed to parse transient PSF file");
    let data = TransientData::from_ascii(&ast);
    assert_eq!(data.signals.len(), 41);
}

#[test]
fn parses_vdiv_sin_ascii() {
    let ast = parse(VDIV_SIN_PSF).expect("Failed to parse transient PSF file");
    let data = TransientData::from_ascii(&ast);
    assert_eq!(data.signals.len(), 4);
    assert_eq!(
        data.signal("time")
            .expect("should contain a time signal")
            .len(),
        16001
    );
}

#[test]
fn parses_ac() {
    let ast = parse(AC_EXAMPLE_PSF).expect("Failed to parse ac PSF file");
    let data = AcData::from_ascii(&ast);
    assert_eq!(data.signals.len(), 3);
    assert_eq!(data.freq.len(), 13);
}

#[test]
fn parses_dc_1() {
    let ast = parse(DC_EXAMPLE1_PSF).expect("Failed to parse dc PSF file");
    let data = DcData::from_ascii(&ast);
    if let DcData::Sweep(data) = data {
        assert_eq!(data.signals.len(), 3);
        assert_eq!(data.sweep_var, "vddval");
        assert_eq!(data.sweep_values.len(), 2);
    } else {
        panic!("expected sweep data, not op data");
    }
}

#[test]
fn parses_dc_2() {
    let ast = parse(DC_EXAMPLE2_PSF).expect("Failed to parse dc PSF file");
    let data = DcData::from_ascii(&ast);
    if let DcData::Op(data) = data {
        assert_eq!(data.signals.len(), 3);
    } else {
        panic!("expected op data, not sweep data");
    }
}
