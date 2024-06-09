use super::*;

use crate::netlist::NetlistOptions;
use crate::Primitive;
use std::path::PathBuf;
use substrate::schematic::netlist::ConvertibleNetlister;

pub const TEST_DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/data");

pub const SPICE_MOS: &str = r#"
.subckt my_mos d g s b
M0 d g s b my_mos_model
.ends
"#;

pub const SPICE_RESISTOR: &str = r#"
.subckt my_resistor p n
R1 p n 100
.ends
"#;

#[inline]
pub fn test_data(file_name: &str) -> PathBuf {
    PathBuf::from(TEST_DATA_DIR).join(file_name)
}

#[test]
fn spice_resistor_tokens() {
    let tok = Tokenizer::new(Dialect::Spice, SPICE_RESISTOR);
    let toks = tok.into_iter().collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(
        toks,
        vec![
            Token::Directive(Substr(".subckt".into())),
            Token::Ident(Substr("my_resistor".into())),
            Token::Ident(Substr("p".into())),
            Token::Ident(Substr("n".into())),
            Token::LineEnd,
            Token::Ident(Substr("R1".into())),
            Token::Ident(Substr("p".into())),
            Token::Ident(Substr("n".into())),
            Token::Ident(Substr("100".into())),
            Token::LineEnd,
            Token::Directive(Substr(".ends".into())),
            Token::LineEnd,
        ]
    );
}

#[test]
fn parse_dff() {
    let parsed = Parser::parse_file(Dialect::Spice, test_data("spice/dff.spice")).unwrap();
    assert_eq!(parsed.ast.elems.len(), 1);
    match &parsed.ast.elems[0] {
        Elem::Subckt(Subckt {
            name,
            ports,
            components,
            connects: _,
        }) => {
            assert_eq!(*name, "openram_dff".into());
            assert_eq!(
                *ports,
                vec![
                    "VDD".into(),
                    "GND".into(),
                    "CLK".into(),
                    "D".into(),
                    "Q".into(),
                    "Q_N".into()
                ]
            );

            let c = &components[10];
            match c {
                Component::Instance(inst) => {
                    assert_eq!(inst.name, "X10".into());
                    assert_eq!(inst.child, "sky130_fd_pr__pfet_01v8".into());
                    assert_eq!(
                        inst.ports,
                        vec![
                            "a_547_712#".into(),
                            "a_28_102#".into(),
                            "VDD".into(),
                            "VDD".into()
                        ]
                    );
                    assert_eq!(
                        inst.params,
                        Params {
                            values: HashMap::from_iter([
                                ("w".into(), "3".into()),
                                ("l".into(), "0.15".into())
                            ]),
                        }
                    );
                }
                _ => panic!("match failed"),
            }
        }
        _ => panic!("match failed"),
    }
}

#[test]
fn parse_pex_netlist() {
    let parsed = Parser::parse_file(Dialect::Spice, test_data("spice/pex_netlist.spice")).unwrap();
    assert_eq!(parsed.ast.elems.len(), 1);
    match &parsed.ast.elems[0] {
        Elem::Subckt(Subckt {
            name,
            ports,
            components,
            connects: _,
        }) => {
            assert_eq!(*name, "sram22_512x64m4w8".into());
            assert!(ports.contains(&"VDD".into()));
            assert!(ports.contains(&"WE".into()));
            assert!(ports.contains(&"VSS".into()));
            assert!(ports.contains(&"CLK".into()));
            assert!(ports.contains(&"WMASK[1]".into()));
            assert!(ports.contains(&"ADDR[2]".into()));
            assert!(ports.contains(&"DIN[63]".into()));
            assert!(ports.contains(&"DIN[22]".into()));
            assert!(ports.contains(&"DIN[1]".into()));
            assert!(ports.contains(&"DOUT[31]".into()));
            assert!(ports.contains(&"DOUT[63]".into()));
            assert!(ports.contains(&"DOUT[0]".into()));
            assert_eq!(components.len(), 12);
        }
        _ => panic!("match failed"),
    }
}

#[test]
fn convert_mos_to_scir() {
    let parsed = Parser::parse(Dialect::Spice, SPICE_MOS).unwrap();
    let converter = ScirConverter::new(&parsed.ast);
    let lib = converter.convert().unwrap();
    let issues = lib.validate();
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);
    assert_eq!(lib.cells().count(), 1);
    let cell = lib.cell_named("my_mos");
    assert_eq!(cell.instances().count(), 1);

    let (_, inst) = cell.instances().next().unwrap();
    let prim = lib.primitive(inst.child().unwrap_primitive());
    match prim {
        Primitive::Mos { model, params } => {
            assert_eq!(model, "my_mos_model");
            assert_eq!(params.len(), 0);
        }
        _ => panic!("incorrect primitive kind"),
    }
}

#[test]
fn convert_dff_to_scir() {
    let parsed = Parser::parse_file(Dialect::Spice, test_data("spice/dff.spice")).unwrap();
    let mut converter = ScirConverter::new(&parsed.ast);
    converter.blackbox("sky130_fd_pr__nfet_01v8");
    converter.blackbox("sky130_fd_pr__pfet_01v8");
    let lib = converter.convert().unwrap();
    let issues = lib.validate();
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);
    assert_eq!(lib.cells().count(), 1);
    let cell = lib.cell_named("openram_dff");
    assert_eq!(cell.instances().count(), 22);

    let (_, inst) = cell.instances().nth(10).unwrap();
    let prim = lib.primitive(inst.child().unwrap_primitive());
    match prim {
        Primitive::RawInstance {
            ports,
            cell,
            params,
        } => {
            assert_eq!(ports.len(), 4);
            assert_eq!(cell, "sky130_fd_pr__pfet_01v8");
            assert_eq!(params.len(), 2);
        }
        _ => panic!("incorrect primitive kind"),
    }
}

#[test]
fn convert_blackbox_to_scir() {
    let parsed = Parser::parse_file(Dialect::Spice, test_data("spice/blackbox.spice")).unwrap();
    let mut converter = ScirConverter::new(&parsed.ast);
    converter.blackbox("blackbox1");
    converter.blackbox("blackbox2");
    let lib = converter.convert().unwrap();
    let issues = lib.validate();
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);
    assert_eq!(lib.cells().count(), 1);
    let cell = lib.cell_named("top");
    assert_eq!(cell.instances().count(), 4);

    let (_, inst) = cell.instances().nth(2).unwrap();
    let prim = lib.primitive(inst.child().unwrap_primitive());
    match prim {
        Primitive::RawInstance {
            ports,
            cell,
            params,
        } => {
            assert_eq!(ports.len(), 2);
            assert_eq!(cell, "blackbox2");
            assert_eq!(params.len(), 0);
        }
        _ => panic!("incorrect primitive kind"),
    }
}

#[test]
fn convert_cdl_rdac_to_scir() {
    let parsed = Parser::parse_file(Dialect::Cdl, test_data("spice/AA_rdac.cdl")).unwrap();
    let mut converter = ScirConverter::new(&parsed.ast);
    converter.blackbox("poly_resistor");
    let lib = converter.convert().unwrap();
    let issues = lib.validate();
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);
    assert_eq!(lib.cells().count(), 25);
    let cell = lib.cell_named("AA_rdac");
    assert_eq!(cell.instances().count(), 3);
    let cell = lib.cell_named("AA_rdac_inv");
    println!("{cell:#?}");
    assert_eq!(cell.instances().count(), 2);
    Spice
        .write_scir_netlist_to_file(
            &lib,
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/build/convert_cdl_rdac_to_scir/converted_spice.sp"
            ),
            NetlistOptions::default(),
        )
        .expect("failed to export SPICE");
}
