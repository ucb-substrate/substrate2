//! SPICE netlist parser.

#[cfg(test)]
mod tests;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use arcstr::ArcStr;
use nom::bytes::complete::{take_till, take_while};
use nom::error::ErrorKind;
use nom::{IResult, InputTakeAtPosition};
use thiserror::Error;

pub type Node = Substr;

#[derive(Clone, Default, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Substr(arcstr::Substr);

#[derive(Clone, Default, Eq, PartialEq, Debug)]
pub struct Parser {
    buffer: Vec<Token>,
    ast: Ast,
    state: ParseState,
}

#[derive(Clone, Default, Eq, PartialEq, Debug)]
enum ParseState {
    #[default]
    Top,
    Subckt(Subckt),
}

impl Parser {
    pub fn parse_file(path: impl AsRef<Path>) -> Result<Ast, ParserError> {
        let path = path.as_ref();
        let s: ArcStr = std::fs::read_to_string(path).unwrap().into();
        let s = Substr(arcstr::Substr::full(s));
        let mut parser = Self::default();
        parser.parse(s)?;
        Ok(parser.ast)
    }

    pub fn parse(&mut self, data: Substr) -> Result<(), ParserError> {
        let mut tok = Tokenizer::new(data);
        while let Some(line) = self.parse_line(&mut tok)? {
            match (&mut self.state, line) {
                (ParseState::Top, Line::SubcktDecl { name, ports }) => {
                    self.state = ParseState::Subckt(Subckt {
                        name,
                        ports,
                        components: vec![],
                    });
                }
                (ParseState::Top, Line::Component(c)) => {
                    self.ast.elems.push(Elem::Component(c));
                }
                (ParseState::Subckt(ref mut subckt), Line::Component(c)) => {
                    subckt.components.push(c);
                }
                (ParseState::Subckt(ref mut subckt), Line::EndSubckt) => {
                    let subckt = std::mem::take(subckt);
                    self.ast.elems.push(Elem::Subckt(subckt));
                    self.state = ParseState::Top;
                }
                (_, line) => return Err(ParserError::UnexpectedLine(Box::new(line))),
            }
        }
        Ok(())
    }

    fn parse_line(&mut self, tok: &mut Tokenizer) -> Result<Option<Line>, ParserError> {
        while let Some(token) = tok.get()? {
            if token == Token::LineEnd {
                return Ok(Some(self.parse_line_inner()?));
            } else {
                self.buffer.push(token);
            }
        }

        Ok(None)
    }

    fn parse_line_inner(&mut self) -> Result<Line, ParserError> {
        let line = match self.buffer.first().unwrap() {
            Token::Directive(d) => {
                if d.eq_ignore_ascii_case(".subckt") {
                    // TODO params
                    let name = self.buffer[1].try_ident()?.clone();
                    let ports = self.buffer[2..]
                        .iter()
                        .map(|tok| tok.try_ident().map(Clone::clone))
                        .collect::<Result<_, _>>()?;
                    Line::SubcktDecl { name, ports }
                } else if d.eq_ignore_ascii_case(".ends") {
                    Line::EndSubckt
                } else {
                    return Err(ParserError::UnexpectedDirective(d.clone()));
                }
            }
            Token::Ident(id) => {
                let kind = id.chars().next().unwrap().to_ascii_uppercase();

                match kind {
                    'R' => Line::Component(Component::Res(Res {
                        name: self.buffer[0].try_ident()?.substr(1..).clone().into(),
                        pos: self.buffer[1].try_ident()?.clone(),
                        neg: self.buffer[2].try_ident()?.clone(),
                        value: self.buffer[3].try_ident()?.clone(),
                    })),
                    'X' => {
                        let pos = self.buffer.iter().position(|t| matches!(t, Token::Equals));
                        let child_idx = pos.unwrap_or(self.buffer.len() + 1) - 2;
                        let child = self.buffer[child_idx].try_ident()?.clone();
                        let ports = self.buffer[1..child_idx]
                            .iter()
                            .map(|x| x.try_ident().map(Clone::clone))
                            .collect::<Result<_, _>>()?;

                        let mut params = Params::default();
                        for i in (child_idx + 1..self.buffer.len()).step_by(3) {
                            let k = self.buffer[i].try_ident()?.clone();
                            assert!(matches!(self.buffer[i + 1], Token::Equals));
                            let v = self.buffer[i + 2].try_ident()?.clone();
                            params.insert(k, v);
                        }

                        Line::Component(Component::Instance(Instance {
                            name: self.buffer[0].try_ident()?.substr(1..).clone().into(),
                            ports,
                            child,
                            params,
                        }))
                    }
                    kind => return Err(ParserError::UnexpectedComponentType(kind)),
                }
            }
            tok => return Err(ParserError::UnexpectedToken(tok.clone())),
        };
        self.buffer.clear();
        Ok(line)
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Ast {
    /// The list of elements in the SPICE netlist.
    elems: Vec<Elem>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Line {
    SubcktDecl { name: Substr, ports: Vec<Substr> },
    Component(Component),
    EndSubckt,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Elem {
    /// A subcircuit declaration.
    Subckt(Subckt),
    /// A top-level component instance.
    Component(Component),
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Subckt {
    name: Substr,
    ports: Vec<Substr>,
    /// List of components in the subcircuit.
    components: Vec<Component>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Component {
    Mos(Mos),
    Res(Res),
    Instance(Instance),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Res {
    name: Substr,
    pos: Node,
    neg: Node,
    value: Substr,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Instance {
    name: Substr,
    ports: Vec<Substr>,
    child: Substr,
    params: Params,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Mos {
    name: Substr,
    /// The drain.
    d: Node,
    /// The gate.
    g: Node,
    /// The source.
    s: Node,
    /// The body/substrate.
    b: Node,
    /// Parameters and their values.
    params: Params,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Params {
    values: HashMap<Substr, Substr>,
}

#[inline]
fn is_newline(c: char) -> bool {
    c == '\n' || c == '\r'
}

#[inline]
fn is_space(c: char) -> bool {
    c == ' ' || c == '\t'
}

#[inline]
fn is_space_or_newline(c: char) -> bool {
    is_space(c) || is_newline(c)
}

#[inline]
fn is_special(c: char) -> bool {
    is_space_or_newline(c) || c == '='
}

pub struct Tokenizer {
    data: Substr,
    rem: Substr,
    state: TokState,
    comment: char,
    line_continuation: char,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token {
    Directive(Substr),
    Ident(Substr),
    LineEnd,
    Equals,
}

#[derive(Copy, Clone, Default, Eq, PartialEq, Hash, Debug)]
enum TokState {
    /// Initial state.
    #[default]
    Init,
    /// Parsing a line.
    Line,
}

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("tokenizer error: {0}")]
    Tokenizer(#[from] TokenizerError),
    #[error("unexpected line: {0:?}")]
    UnexpectedLine(Box<Line>),
    #[error("unexpected SPICE directive: {0}")]
    UnexpectedDirective(Substr),
    #[error("unexpected component type: {0}")]
    UnexpectedComponentType(char),
    #[error("unexpected token: {0:?}")]
    UnexpectedToken(Token),
}

#[derive(Debug, Error)]
#[allow(dead_code)]
pub struct TokenizerError {
    state: TokState,
    ofs: usize,
    data: Substr,
    message: ArcStr,
    token: Substr,
}

impl Tokenizer {
    pub fn new(data: impl Into<arcstr::Substr>) -> Self {
        let data = data.into();
        let rem = data.clone();
        Self {
            data: Substr(data),
            rem: Substr(rem),
            state: TokState::Init,
            comment: '*',
            line_continuation: '+',
        }
    }

    pub fn get(&mut self) -> Result<Option<Token>, TokenizerError> {
        loop {
            self.take_ws();
            if self.rem.is_empty() {
                if self.state == TokState::Line {
                    self.state = TokState::Init;
                    return Ok(Some(Token::LineEnd));
                } else {
                    return Ok(None);
                }
            }

            let c = self.peek().unwrap();
            if c == '=' {
                self.take1();
                return Ok(Some(Token::Equals));
            }
            match self.state {
                TokState::Init => {
                    if c == self.comment {
                        self.take_until_newline();
                    } else if c.is_whitespace() {
                        self.take1();
                    } else if c == self.line_continuation {
                        self.err("unexpected line continuation", c)?;
                    } else {
                        self.state = TokState::Line;
                    }
                }
                TokState::Line => {
                    if is_newline(c) {
                        self.take_ws();
                        if self.peek().unwrap_or(self.line_continuation) != self.line_continuation {
                            self.state = TokState::Init;
                            return Ok(Some(Token::LineEnd));
                        }
                    } else if c == self.line_continuation {
                        self.take1();
                    } else if c == self.comment {
                        self.take_until_newline();
                    } else if c == '.' {
                        let word = self.take_ident();
                        return Ok(Some(Token::Directive(word)));
                    } else {
                        let word = self.take_ident();
                        return Ok(Some(Token::Ident(word)));
                    }
                }
            }
        }
    }

    fn err(
        &self,
        message: impl Into<ArcStr>,
        token: impl Into<Substr>,
    ) -> Result<(), TokenizerError> {
        Err(TokenizerError {
            state: self.state,
            ofs: self.rem.range().start,
            data: self.data.clone(),
            message: message.into(),
            token: token.into(),
        })
    }

    fn take1(&mut self) -> Option<char> {
        let c = self.rem.chars().next()?;
        self.rem = Substr(self.rem.substr(1..));
        Some(c)
    }

    fn take_until_newline(&mut self) -> Substr {
        let (rest, comment) = take_till::<_, _, ()>(is_newline)(self.rem.clone()).unwrap();
        self.rem = rest;
        comment
    }

    fn take_ident(&mut self) -> Substr {
        let (rest, value) = take_till::<_, _, ()>(is_special)(self.rem.clone()).unwrap();
        self.rem = rest;
        value
    }

    fn take_ws(&mut self) {
        let (rest, _) = take_while::<_, _, ()>(is_space)(self.rem.clone()).unwrap();
        self.rem = rest;
    }

    fn peek(&self) -> Option<char> {
        self.rem.chars().next()
    }
}

pub struct Tokens {
    tok: Tokenizer,
}

impl Iterator for Tokens {
    type Item = Result<Token, TokenizerError>;
    fn next(&mut self) -> Option<Self::Item> {
        self.tok.get().transpose()
    }
}

impl IntoIterator for Tokenizer {
    type Item = Result<Token, TokenizerError>;
    type IntoIter = Tokens;
    fn into_iter(self) -> Self::IntoIter {
        Tokens { tok: self }
    }
}

impl Deref for Substr {
    type Target = arcstr::Substr;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Substr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl InputTakeAtPosition for Substr {
    type Item = char;
    fn split_at_position<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        <&str as InputTakeAtPosition>::split_at_position::<P, (&str, ErrorKind)>(
            &&***self, predicate,
        )
        .map(|(i, o)| (Substr(self.0.substr_from(i)), Substr(self.0.substr_from(o))))
        .map_err(|e| e.map(|e| E::from_error_kind(self.clone(), e.1)))
    }
    fn split_at_position1<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
        e: nom::error::ErrorKind,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        <&str as InputTakeAtPosition>::split_at_position1::<P, (&str, ErrorKind)>(
            &&***self, predicate, e,
        )
        .map(|(i, o)| (Substr(self.0.substr_from(i)), Substr(self.0.substr_from(o))))
        .map_err(|e| e.map(|e| E::from_error_kind(self.clone(), e.1)))
    }
    fn split_at_position_complete<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        <&str as InputTakeAtPosition>::split_at_position_complete::<P, (&str, ErrorKind)>(
            &&***self, predicate,
        )
        .map(|(i, o)| (Substr(self.0.substr_from(i)), Substr(self.0.substr_from(o))))
        .map_err(|e| e.map(|e| E::from_error_kind(self.clone(), e.1)))
    }
    fn split_at_position1_complete<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
        e: nom::error::ErrorKind,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        <&str as InputTakeAtPosition>::split_at_position1_complete::<P, (&str, ErrorKind)>(
            &&***self, predicate, e,
        )
        .map(|(i, o)| (Substr(self.0.substr_from(i)), Substr(self.0.substr_from(o))))
        .map_err(|e| e.map(|e| E::from_error_kind(self.clone(), e.1)))
    }
}

impl Display for Substr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Substr> for arcstr::Substr {
    fn from(value: Substr) -> Self {
        value.0
    }
}

impl From<&str> for Substr {
    fn from(value: &str) -> Self {
        Self(arcstr::Substr::from(value))
    }
}

impl From<arcstr::Substr> for Substr {
    fn from(value: arcstr::Substr) -> Self {
        Self(value)
    }
}

impl From<char> for Substr {
    fn from(value: char) -> Self {
        Self(arcstr::Substr::from(value.to_string()))
    }
}

impl Token {
    pub fn unwrap_ident(&self) -> &Substr {
        match self {
            Self::Ident(x) => x,
            _ => panic!("not an ident"),
        }
    }
    pub fn try_ident(&self) -> Result<&Substr, ParserError> {
        match self {
            Self::Ident(x) => Ok(x),
            _ => Err(ParserError::UnexpectedToken(self.clone())),
        }
    }
}

impl Params {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, k: impl Into<Substr>, v: impl Into<Substr>) {
        self.values.insert(k.into(), v.into());
    }

    pub fn get(&self, k: &str) -> Option<&Substr> {
        self.values.get(k)
    }
}

impl Borrow<str> for Substr {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Display for TokenizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (token {} at offset {})",
            self.message, self.token, self.ofs
        )
    }
}
