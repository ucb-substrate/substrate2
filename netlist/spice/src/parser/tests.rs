use super::*;

const SPICE_RESISTOR: &str = r#"
.subckt my_resistor p n
R1 p n 100
.ends
"#;

#[test]
fn spice_resistor_tokens() {
    let tok = Tokenizer::new(SPICE_RESISTOR);
    let toks = tok.into_iter().collect::<Vec<_>>();
    assert_eq!(
        toks,
        vec![
            Token::Directive(".subckt".into()),
            Token::Ident("my_resistor".into()),
            Token::Ident("p".into()),
            Token::Ident("n".into()),
            Token::LineEnd,
            Token::Ident("R1".into()),
            Token::Ident("p".into()),
            Token::Ident("n".into()),
            Token::Ident("100".into()),
            Token::LineEnd,
            Token::Directive(".ends".into()),
            Token::LineEnd,
        ]
    );
}
