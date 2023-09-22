use std::path::PathBuf;

use crate::parse;

use super::*;

pub(crate) const EXAMPLES_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples");

const VARIABLE: &str = " 0	v(xdut.vdd)	voltage\r\n";

const VARIABLES: &str = r###"Variables:
	0	v(xdut.vdd)	voltage
	1	v(xdut.out)	voltage
	2	i(v.xdut.vdd)	current

"###;

#[test]
fn test_variables() {
    let (_, vars) = variables(VARIABLES.as_bytes()).unwrap();
    println!("{vars:?}");
    assert_eq!(vars.len(), 3);
}

#[test]
fn test_variable() {
    let (_, var) = variable(VARIABLE.as_bytes()).unwrap();
    println!("{var:?}");
    assert_eq!(var.idx, 0);
}

#[test]
fn test_binary_analysis() {
    let path = PathBuf::from(EXAMPLES_PATH).join("rawspice_binary.raw");
    let data = std::fs::read(path).unwrap();
    let (_, analysis) = analysis(Options::default())(&data).unwrap();
    println!("{analysis:?}");

    assert_eq!(analysis.num_variables, 4);
    assert_eq!(analysis.num_points, 301);
    assert_eq!(analysis.variables.len(), 4);

    let data = analysis.data.unwrap_complex();
    assert_eq!(data.len(), 4);
    assert_eq!(data[0].real.len(), 301);
    assert_eq!(data[0].imag.len(), 301);
}

#[test]
fn test_binary_analyses() {
    let path = PathBuf::from(EXAMPLES_PATH).join("rawspice_binary.raw");
    let data = std::fs::read(path).unwrap();
    let (_, mut analyses) = analyses(&data, Options::default()).unwrap();
    println!("{analyses:?}");

    assert_eq!(analyses.len(), 3);

    let analyses2 = analyses.pop().unwrap();
    assert_eq!(analyses2.num_variables, 4);
    assert_eq!(analyses2.num_points, 1008);
    let data2 = analyses2.data.unwrap_real();
    assert_eq!(data2.len(), 4);
    assert_eq!(data2[1].len(), 1008);

    let analyses1 = analyses.pop().unwrap();
    assert_eq!(analyses1.num_variables, 3);
    assert_eq!(analyses1.num_points, 1);
    let data1 = analyses1.data.unwrap_real();
    assert_eq!(data1.len(), 3);
    assert_eq!(data1[1].len(), 1);

    let analyses0 = analyses.pop().unwrap();
    assert_eq!(analyses0.num_variables, 4);
    assert_eq!(analyses0.num_points, 301);
    let data0 = analyses0.data.unwrap_complex();
    assert_eq!(data0.len(), 4);
    assert_eq!(data0[0].real.len(), 301);
    assert_eq!(data0[0].imag.len(), 301);
}

#[test]
fn test_ascii_analysis() {
    let path = PathBuf::from(EXAMPLES_PATH).join("rawspice_ascii.raw");
    let data = std::fs::read(path).unwrap();
    let (_, analysis) = analysis(Options::default())(&data).unwrap();
    println!("{analysis:?}");

    assert_eq!(analysis.num_variables, 4);
    assert_eq!(analysis.num_points, 13);
    assert_eq!(analysis.variables.len(), 4);

    let data = analysis.data.unwrap_complex();
    assert_eq!(data.len(), 4);
    assert_eq!(data[0].real.len(), 13);
    assert_eq!(data[0].imag.len(), 13);
}

#[test]
fn test_ascii_analyses() {
    let path = PathBuf::from(EXAMPLES_PATH).join("rawspice_ascii.raw");
    let data = std::fs::read(path).unwrap();
    let (_, mut analyses) = analyses(&data, Options::default()).unwrap();
    println!("{analyses:?}");

    assert_eq!(analyses.len(), 4);

    let analyses3 = analyses.pop().unwrap();
    assert_eq!(analyses3.num_variables, 4);
    assert_eq!(analyses3.num_points, 59);
    let data3 = analyses3.data.unwrap_real();
    assert_eq!(data3.len(), 4);
    assert_eq!(data3[0].len(), 59);

    let analyses2 = analyses.pop().unwrap();
    assert_eq!(analyses2.num_variables, 3);
    assert_eq!(analyses2.num_points, 1);
    let data2 = analyses2.data.unwrap_real();
    assert_eq!(data2.len(), 3);
    assert_eq!(data2[1].len(), 1);

    let analyses1 = analyses.pop().unwrap();
    assert_eq!(analyses1.num_variables, 4);
    assert_eq!(analyses1.num_points, 6);
    let data1 = analyses1.data.unwrap_real();
    assert_eq!(data1.len(), 4);
    assert_eq!(data1[1].len(), 6);

    let analyses0 = analyses.pop().unwrap();
    assert_eq!(analyses0.num_variables, 4);
    assert_eq!(analyses0.num_points, 13);
    let data0 = analyses0.data.unwrap_complex();
    assert_eq!(data0.len(), 4);
    assert_eq!(data0[1].real.len(), 13);
    assert_eq!(data0[1].imag.len(), 13);
}

#[test]
fn test_vdivider_analyses() {
    for path in ["netlist.ascii.raw", "netlist.bin.raw"] {
        let path = PathBuf::from(EXAMPLES_PATH).join(path);
        let data = std::fs::read(path).unwrap();
        let rawfile = parse(&data, Options::default()).unwrap();

        let analysis = &rawfile.analyses[0];
        assert_eq!(analysis.num_variables, 5);
        assert_eq!(analysis.num_points, 55);
        assert_eq!(analysis.variables.len(), 5);

        let data = analysis.data.as_ref().unwrap_real();
        assert_eq!(data.len(), 5);
        data.iter().for_each(|vec| assert_eq!(vec.len(), 55));

        let analysis = &rawfile.analyses[1];
        assert_eq!(analysis.num_variables, 5);
        assert_eq!(analysis.num_points, 102);
        assert_eq!(analysis.variables.len(), 5);

        let data = analysis.data.as_ref().unwrap_complex();
        assert_eq!(data.len(), 5);
        data.iter().for_each(|vec| {
            assert_eq!(vec.real.len(), 102);
            assert_eq!(vec.imag.len(), 102);
        });

        let analysis = &rawfile.analyses[2];
        assert_eq!(analysis.num_variables, 5);
        assert_eq!(analysis.num_points, 51);
        assert_eq!(analysis.variables.len(), 5);

        let data = analysis.data.as_ref().unwrap_real();
        assert_eq!(data.len(), 5);
        data.iter().for_each(|vec| assert_eq!(vec.len(), 51));

        let analysis = &rawfile.analyses[3];
        assert_eq!(analysis.num_variables, 6);
        assert_eq!(analysis.num_points, 102);
        assert_eq!(analysis.variables.len(), 6);

        let data = analysis.data.as_ref().unwrap_real();
        assert_eq!(data.len(), 6);
        data.iter().for_each(|vec| assert_eq!(vec.len(), 102));
    }
}

#[test]
fn test_vdivider2_analyses() {
    for path in ["netlist2.ascii.raw", "netlist2.bin.raw"] {
        println!("Parsing {path}");
        let path = PathBuf::from(EXAMPLES_PATH).join(path);
        let data = std::fs::read(path).unwrap();
        let rawfile = parse(&data, Options::default()).unwrap();
        println!("Rawfile: {rawfile:?}");

        let analysis = &rawfile.analyses[0];
        assert_eq!(analysis.num_variables, 4);
        assert_eq!(analysis.num_points, 104);
        assert_eq!(analysis.variables.len(), 4);

        let data = analysis.data.as_ref().unwrap_real();
        assert_eq!(data.len(), 4);
        data.iter().for_each(|vec| assert_eq!(vec.len(), 104));
        let var = analysis
            .variables
            .iter()
            .find(|x| x.name == "xinst0_n")
            .unwrap();
        let vec = &data[var.idx];
        vec.iter()
            .for_each(|f| assert!(approx::relative_eq!(*f, 1.2)));
    }
}

#[test]
fn test_ngspice_analyses() {
    let path = "ngspice.bin.raw";
    println!("Parsing {path}");
    let path = PathBuf::from(EXAMPLES_PATH).join(path);
    let data = std::fs::read(path).unwrap();
    let rawfile = parse(
        &data,
        Options {
            endianness: ByteOrder::LittleEndian,
        },
    )
    .unwrap();
    println!("Rawfile: {rawfile:?}");

    let analysis = &rawfile.analyses[0];
    assert_eq!(analysis.num_variables, 6);
    assert_eq!(analysis.num_points, 59);
    assert_eq!(analysis.variables.len(), 6);

    let data = analysis.data.as_ref().unwrap_real();
    assert_eq!(data.len(), 6);
    data.iter().for_each(|vec| assert_eq!(vec.len(), 59));
    let var = analysis
        .variables
        .iter()
        .find(|x| x.name == "v(xinst0_n)")
        .unwrap();
    let vec = &data[var.idx];
    let expected = 0.6;
    vec.iter().for_each(|f| {
        assert!(
            approx::relative_eq!(*f, expected),
            "expected {expected}, found {f}"
        )
    });
}
