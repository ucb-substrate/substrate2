use super::*;

use crate::PrimitiveKind;
use std::path::PathBuf;

pub const TEST_DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/data");

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
    let tok = Tokenizer::new(SPICE_RESISTOR);
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
    let parsed = Parser::parse_file(test_data("spice/dff.spice")).unwrap();
    assert_eq!(parsed.ast.elems.len(), 1);
    match &parsed.ast.elems[0] {
        Elem::Subckt(Subckt {
            name,
            ports,
            components,
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
fn convert_dff_to_scir() {
    let parsed = Parser::parse_file(test_data("spice/dff.spice")).unwrap();
    let converter = ScirConverter::new("openram_dff", &parsed.ast);
    let lib = converter.convert().unwrap();
    let issues = lib.validate();
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);
    assert_eq!(lib.cells().count(), 1);
    let cell = lib.cell_named("openram_dff");
    assert_eq!(cell.instances().count(), 22);

    let (_, inst) = cell.instances().nth(10).unwrap();
    let prim = lib.primitive(inst.child().unwrap_primitive());
    match &prim.kind {
        PrimitiveKind::RawInstance { ports, cell } => {
            assert_eq!(ports.len(), 4);
            assert_eq!(cell, "sky130_fd_pr__pfet_01v8");
            assert_eq!(prim.params.len(), 2);
        }
        _ => panic!("incorrect primitive kind"),
    }
}

#[test]
fn convert_blackbox_to_scir() {
    let parsed = Parser::parse_file(test_data("spice/blackbox.spice")).unwrap();
    let mut converter = ScirConverter::new("top", &parsed.ast);
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
    match &prim.kind {
        PrimitiveKind::RawInstance { ports, cell } => {
            assert_eq!(ports.len(), 2);
            assert_eq!(cell, "blackbox2");
            assert_eq!(prim.params.len(), 0);
        }
        _ => panic!("incorrect primitive kind"),
    }
}
