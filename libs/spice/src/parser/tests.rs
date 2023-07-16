use super::*;

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
    let ast = Parser::parse_file(test_data("spice/dff.spice")).unwrap();
    assert_eq!(ast.elems.len(), 1);
    match &ast.elems[0] {
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
                    assert_eq!(inst.name, "10".into());
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
