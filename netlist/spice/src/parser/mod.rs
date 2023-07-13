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

pub struct Parser {}

impl Parser {
    pub fn parse_file() {}

    pub fn parse(data: Substr) {}

    fn parse_inner(data: Substr, ast: &mut Ast) {}
}

pub struct Ast {
    elems: Vec<Elem>,
}

pub enum Elem {
    Subckt(Subckt),
    Component(Component),
}
pub struct Subckt {
    components: Vec<Component>,
}

pub enum Component {
    Mos(Mos),
}

pub struct Mos {
    name: Substr,
    d: Node,
    g: Node,
    s: Node,
    b: Node,
    params: Params,
}

pub struct Params {
    values: HashMap<Substr, Substr>,
}

fn is_newline(c: char) -> bool {
    c == '\n' || c == '\r'
}

fn is_space(c: char) -> bool {
    c == ' ' || c == '\t'
}

fn is_space_or_newline(c: char) -> bool {
    is_space(c) || is_newline(c)
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
                        let word = self.take_until_ws();
                        return Some(Token::Directive(word));
                    } else {
                        let word = self.take_until_ws();
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

    fn take_until_ws(&mut self) -> Substr {
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
    T: Into<arcstr::Substr>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl Display for Substr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
