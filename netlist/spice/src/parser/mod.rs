//! SPICE netlist parser.

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::fmt::Display;
use std::ops::{Deref, DerefMut};

use nom::bytes::complete::{take_till, take_while};
use nom::error::ErrorKind;
use nom::{IResult, InputTakeAtPosition};

pub type Node = Substr;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Substr(arcstr::Substr);

pub struct Parser {
    buffer: Vec<Token>,
    ast: Ast,
}

impl Parser {
    pub fn parse_file() {}

    pub fn parse(&mut self, data: Substr) {
    }

    fn parse_line(&mut self, data: Substr) {
        let tokens = Tokenizer::new(data);
        for token in tokens {
            if token == Token::LineEnd {
                self.process_buffer();
            } else {
                self.buffer.push(token);
            }
        }
    }

    fn parse_line_inner(&mut self) -> Option<Line> {
        let line = match self.buffer.first().unwrap() {
            Token::Directive(d) => {
                if d.eq_ignore_ascii_case(".subckt") {
                    // TODO params
                    let ports = self.buffer[1..].iter().map(|tok| {
                        tok.unwrap_ident().clone()
                    }).collect();
                    Some(Line::SubcktDecl { ports })
                } else if d.eq_ignore_ascii_case(".ends") {
                    Some(Line::EndSubckt)
                } else {
                    None
                }
            },
            Token::Ident(id) => {
                let pos = self.buffer.iter().position(|t| matches!(t, Token::Equals));
                Some(Line::Component(todo!()))
            },
            tok => panic!("Unexpected token: {:?}", tok),
        };
        self.buffer.clear();
        line
    }
}

pub struct Ast {
    /// The list of elements in the SPICE netlist.
    elems: Vec<Elem>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum Line {
    SubcktDecl {
        ports: Vec<Substr>
    },
    Component(Component),
    EndSubckt,
}

pub enum Elem {
    /// A subcircuit declaration.
    Subckt(Subckt),
    /// A top-level component instance.
    Component(Component),
}

pub struct Subckt {
    /// List of components in the subcircuit.
    components: Vec<Component>,
    ports: Vec<Substr>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Component {
    Mos(Mos),
    Instance(Instance),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Instance {
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

#[derive(Debug, Clone, Eq, PartialEq)]
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

    pub fn get(&mut self) -> Option<Token> {
        println!("GET");
        loop {
            println!("Rem: {}", self.rem);
            self.take_ws();
            if self.rem.is_empty() {
                if self.state == TokState::Line {
                    self.state = TokState::Init;
                    return Some(Token::LineEnd);
                } else {
                    return None;
                }
            }

            let c = self.peek().unwrap();
            if c == '=' {
                return Some(Token::Equals);
            }
            match self.state {
                TokState::Init => {
                    if c == self.comment {
                        self.take_until_newline();
                    } else if c.is_whitespace() {
                        self.take1();
                    } else if c == self.line_continuation {
                        // TODO: error handling
                        panic!("unexpected line continuation");
                    } else {
                        self.state = TokState::Line;
                    }
                }
                TokState::Line => {
                    if is_newline(c) {
                        self.take_ws();
                        if self.peek().unwrap_or(self.line_continuation) != self.line_continuation {
                            self.state = TokState::Init;
                            return Some(Token::LineEnd);
                        }
                    } else if c == self.line_continuation {
                        self.take1();
                    } else if c == self.comment {
                        self.take_until_newline();
                    } else if c == '.' {
                        let word = self.take_ident();
                        return Some(Token::Directive(word));
                    } else {
                        let word = self.take_ident();
                        return Some(Token::Ident(word));
                    }
                }
            }
        }
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
        let (rest, value) = take_till::<_, _, ()>(is_space_or_newline)(self.rem.clone()).unwrap();
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
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        self.tok.get()
    }
}

impl IntoIterator for Tokenizer {
    type Item = Token;
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

impl<T> From<T> for Substr
where
    arcstr::Substr: From<T>,
{
    fn from(value: T) -> Self {
        Self(arcstr::Substr::from(value))
    }
}

impl Display for Substr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Into<arcstr::Substr> for Substr {
    fn into(self) -> arcstr::Substr {
        self.0
    }
}

impl Token {
    pub fn unwrap_ident(&self) -> &Substr {
        match self {
            Self::Ident(x) => x,
            _ => panic!("not an ident"),
        }
    }
}
