//! SPICE netlist parser.

pub mod conv;
pub mod shorts;
#[cfg(test)]
mod tests;

use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

use crate::parser::conv::convert_str_to_numeric_lit;
use crate::Spice;
use arcstr::ArcStr;
use nom::bytes::complete::{take_till, take_while};
use nom::error::ErrorKind;
use nom::{IResult, InputTakeAtPosition};
use thiserror::Error;

use self::conv::ScirConverter;

/// The type representing nodes in a parsed SPICE circuit.
pub type Node = Substr;

/// A substring of a file being parsed.
#[derive(Clone, Default, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Substr(arcstr::Substr);

/// The SPICE dialect to parse.
#[derive(Copy, Clone, Default, Eq, PartialEq, Debug)]
pub enum Dialect {
    /// Vanilla SPICE.
    ///
    /// Selected by default.
    #[default]
    Spice,
    /// CDL.
    Cdl,
}

/// Parses SPICE netlists.
#[derive(Clone, Default, Eq, PartialEq, Debug)]
pub struct Parser {
    dialect: Dialect,
    buffer: Vec<Token>,
    ast: Ast,
    state: ParserState,
}

#[derive(Clone, Default, Eq, PartialEq, Debug)]
struct ParserState {
    include_stack: Vec<PathBuf>,
    reader_state: ReaderState,
}
#[derive(Clone, Default, Eq, PartialEq, Debug)]
enum ReaderState {
    #[default]
    Top,
    Subckt(Subckt),
}

impl Parser {
    /// Makes a new parser for the given SPICE dialect.
    pub fn new(dialect: Dialect) -> Self {
        Self {
            dialect,
            ..Self::default()
        }
    }
    /// Parse the given file.
    pub fn parse_file(
        dialect: Dialect,
        path: impl AsRef<Path>,
    ) -> Result<ParsedSpice, ParserError> {
        let path = path.as_ref();
        tracing::debug!("reading SPICE file: {:?}", path);
        let s: ArcStr = std::fs::read_to_string(path).unwrap().into();
        let s = Substr(arcstr::Substr::full(s));
        let mut parser = Self::new(dialect);
        parser.state.include_stack.push(path.into());
        let name = match s.lines().next() {
            Some(name) => ArcStr::from(name),
            None => arcstr::format!("{:?}", path),
        };
        parser.parse_inner(s)?;

        let parsed = ParsedSpice {
            ast: parser.ast,
            root: Some(path.to_path_buf()),
            name,
        };
        Ok(parsed)
    }

    fn parse_file_inner(&mut self, path: impl AsRef<Path>) -> Result<(), ParserError> {
        let path = path.as_ref();
        let s: ArcStr = std::fs::read_to_string(path)
            .map_err(|err| ParserError::FailedToRead {
                path: path.into(),
                err,
            })?
            .into();
        let s = Substr(arcstr::Substr::full(s));
        self.state.include_stack.push(path.into());
        let res = self.parse_inner(s);
        self.state.include_stack.pop().unwrap();
        res?;
        Ok(())
    }

    /// Parse the given string.
    pub fn parse(dialect: Dialect, data: impl Into<Substr>) -> Result<ParsedSpice, ParserError> {
        let data = data.into();
        let mut parser = Self::new(dialect);
        let name = match data.lines().next() {
            Some(name) => ArcStr::from(name),
            None => arcstr::literal!("spice_library"),
        };
        parser.parse_inner(data)?;

        let parsed = ParsedSpice {
            ast: parser.ast,
            root: None,
            name,
        };
        Ok(parsed)
    }

    fn parse_inner(&mut self, data: Substr) -> Result<(), ParserError> {
        let mut tok = Tokenizer::new(self.dialect, data);
        while let Some(line) = self.parse_line(&mut tok)? {
            match (&mut self.state.reader_state, line) {
                (ReaderState::Top, Line::SubcktDecl { name, ports }) => {
                    self.state.reader_state = ReaderState::Subckt(Subckt {
                        name,
                        ports,
                        components: vec![],
                        connects: vec![],
                    });
                }
                (ReaderState::Top, Line::Component(c)) => {
                    self.ast.elems.push(Elem::Component(c));
                }
                (ReaderState::Top, Line::Include { path }) => {
                    let resolved_path = Path::new::<str>(path.0.as_ref());
                    let resolved_path = if resolved_path.is_relative() {
                        let root = self
                            .state
                            .include_stack
                            .last()
                            .ok_or(ParserError::UnexpectedRelativePath(path.clone()))?;
                        root.parent().unwrap().join(resolved_path)
                    } else {
                        resolved_path.into()
                    };
                    self.parse_file_inner(resolved_path)?;
                }
                (ReaderState::Subckt(ref mut subckt), Line::Component(c)) => {
                    subckt.components.push(c);
                }
                (ReaderState::Subckt(ref mut subckt), Line::Connect { node1, node2 }) => {
                    subckt.connects.push((node1, node2));
                }
                (ReaderState::Subckt(ref mut subckt), Line::EndSubckt) => {
                    let subckt = std::mem::take(subckt);
                    self.ast.elems.push(Elem::Subckt(subckt));
                    self.state.reader_state = ReaderState::Top;
                }
                (_, line) => return Err(ParserError::UnexpectedLine(Box::new(line))),
            }
        }
        Ok(())
    }

    fn parse_line(&mut self, tok: &mut Tokenizer) -> Result<Option<Line>, ParserError> {
        while let Some(token) = tok.get()? {
            if token == Token::LineEnd {
                if let Some(line) = self.parse_line_inner()? {
                    return Ok(Some(line));
                }
            } else {
                self.buffer.push(token);
            }
        }

        Ok(None)
    }

    fn parse_line_inner(&mut self) -> Result<Option<Line>, ParserError> {
        let line = match self.buffer.first().unwrap() {
            Token::Directive(d) => {
                if d.eq_ignore_ascii_case(".subckt") {
                    // TODO params
                    let name = self.buffer[1].try_ident()?.clone();
                    let ports = self.buffer[2..]
                        .iter()
                        .map(|tok| tok.try_ident().cloned())
                        .collect::<Result<_, _>>()?;
                    Line::SubcktDecl { name, ports }
                } else if d.eq_ignore_ascii_case(".ends") {
                    Line::EndSubckt
                } else if d.eq_ignore_ascii_case(".include") {
                    let mut path = self.buffer[1].try_ident()?.clone();
                    // remove enclosing quotation marks, if any.
                    if path.starts_with('"') {
                        let mut chars = path.chars();
                        chars.next().unwrap();
                        chars.next_back().unwrap();
                        path = Substr(path.substr_from(chars.as_str()));
                    }
                    Line::Include { path }
                } else {
                    return Err(ParserError::UnexpectedDirective(d.clone()));
                }
            }
            Token::MetaDirective(d) => {
                if d.eq_ignore_ascii_case("connect") {
                    // TODO: assert buffer length is 3 (connect, node1, node2).
                    if self.buffer.len() != 3 {
                        return Err(ParserError::InvalidLine {
                            line: self.buffer.clone(),
                            reason: "CONNECT statements must specify exactly 2 nodes".to_string(),
                        });
                    }
                    let node1 = self.buffer[1].try_ident()?.clone();
                    let node2 = self.buffer[2].try_ident()?.clone();
                    Line::Connect { node1, node2 }
                } else {
                    // Ignore this line: clear the buffer and return no line
                    self.buffer.clear();
                    return Ok(None);
                }
            }
            Token::Ident(id) => {
                let kind = id.chars().next().unwrap().to_ascii_uppercase();

                match kind {
                    'M' => {
                        let mut params = Params::default();
                        for i in (6..self.buffer.len()).step_by(3) {
                            let k = self.buffer[i].try_ident()?.clone();
                            assert!(matches!(self.buffer[i + 1], Token::Equals));
                            let v = self.buffer[i + 2].try_ident()?.clone();
                            params.insert(k, v);
                        }
                        Line::Component(Component::Mos(Mos {
                            name: self.buffer[0].try_ident()?.clone(),
                            d: self.buffer[1].try_ident()?.clone(),
                            g: self.buffer[2].try_ident()?.clone(),
                            s: self.buffer[3].try_ident()?.clone(),
                            b: self.buffer[4].try_ident()?.clone(),
                            model: self.buffer[5].try_ident()?.clone(),
                            params,
                        }))
                    }
                    'D' => {
                        let mut params = Params::default();
                        for i in (4..self.buffer.len()).step_by(3) {
                            let k = self.buffer[i].try_ident()?.clone();
                            assert!(matches!(self.buffer[i + 1], Token::Equals));
                            let v = self.buffer[i + 2].try_ident()?.clone();
                            params.insert(k, v);
                        }
                        Line::Component(Component::Diode(Diode {
                            name: self.buffer[0].try_ident()?.clone(),
                            pos: self.buffer[1].try_ident()?.clone(),
                            neg: self.buffer[2].try_ident()?.clone(),
                            model: self.buffer[3].try_ident()?.clone(),
                            params,
                        }))
                    }
                    'R' => {
                        let mut params = Params::default();
                        for i in (4..self.buffer.len()).step_by(3) {
                            let k = self.buffer[i].try_ident()?.clone();
                            assert!(matches!(self.buffer[i + 1], Token::Equals));
                            let v = self.buffer[i + 2].try_ident()?.clone();
                            params.insert(k, v);
                        }
                        let value = self.buffer[3].try_ident()?.clone();
                        let value = if convert_str_to_numeric_lit(&value).is_some() {
                            DeviceValue::Value(value)
                        } else {
                            DeviceValue::Model(value)
                        };
                        Line::Component(Component::Res(Res {
                            name: self.buffer[0].try_ident()?.clone(),
                            pos: self.buffer[1].try_ident()?.clone(),
                            neg: self.buffer[2].try_ident()?.clone(),
                            value,
                            params,
                        }))
                    }
                    'C' => Line::Component(Component::Cap(Cap {
                        name: self.buffer[0].try_ident()?.clone(),
                        pos: self.buffer[1].try_ident()?.clone(),
                        neg: self.buffer[2].try_ident()?.clone(),
                        value: self.buffer[3].try_ident()?.clone(),
                    })),
                    'X' => {
                        // An X instance line looks like this:
                        //
                        // ```spice
                        // Xname port0 port1 port2 child param1=value1 param2=value2
                        // ```
                        //
                        // The index of "child" is the index of the first equals sign minus 2.
                        // If there is no equal sign, it is buffer.len() - 1.
                        //
                        // The tokens after Xname and before `child_idx` are ports;
                        // the tokens after `child_idx` should come in groups of 3
                        // and represent parameter values.
                        //
                        // TODO: this logic needs to change to support expressions
                        // in parameter values.
                        let pos = self.buffer.iter().position(|t| matches!(t, Token::Equals));
                        let child_idx = pos.unwrap_or(self.buffer.len() + 1) - 2;
                        let child = self.buffer[child_idx].try_ident()?.clone();
                        let port_end_idx = child_idx;
                        let ports = self.buffer[1..port_end_idx]
                            .iter()
                            .map(|x| x.try_ident().cloned())
                            .collect::<Result<_, _>>()?;

                        let mut params = Params::default();
                        for i in (child_idx + 1..self.buffer.len()).step_by(3) {
                            let k = self.buffer[i].try_ident()?.clone();
                            assert!(matches!(self.buffer[i + 1], Token::Equals));
                            let v = self.buffer[i + 2].try_ident()?.clone();
                            params.insert(k, v);
                        }

                        Line::Component(Component::Instance(Instance {
                            name: self.buffer[0].try_ident()?.clone(),
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
        Ok(Some(line))
    }
}

/// Data associated with parsing a SPICE file.
pub struct ParsedSpice {
    /// The parsed contents of the spice file.
    pub ast: Ast,

    /// The file path at the root of the `include` tree.
    pub root: Option<PathBuf>,

    /// The name of the netlist.
    ///
    /// By default, this is the first line of the root file,
    /// with whitespace trimmed.
    pub name: ArcStr,
}

/// The abstract syntax tree (AST) of a parsed SPICE netlist.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Ast {
    /// The list of elements in the SPICE netlist.
    pub elems: Vec<Elem>,
}

/// A single logical line in a SPICE netlist.
///
/// A logical line may contain multiple lines in a file
/// if all lines after the first are separated by the line continuation
/// character (typically '+').
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Line {
    /// A subcircuit declaration.
    SubcktDecl {
        /// The name of the subcircuit.
        name: Substr,
        /// A list of ports.
        ///
        /// Each port is the name of a node exposed by the subcircuit.
        ports: Vec<Node>,
    },
    /// A component instantiation.
    Component(Component),
    /// The end of a subcircuit.
    EndSubckt,
    /// An include directive.
    Include {
        /// The path to include.
        path: Substr,
    },
    /// Connect (i.e. deep short) two nodes.
    Connect {
        /// The first node.
        node1: Substr,
        /// The second node.
        node2: Substr,
    },
}

/// An element of a SPICE netlist AST.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Elem {
    /// A subcircuit declaration.
    Subckt(Subckt),
    /// A top-level component instance.
    Component(Component),
}

/// The contents of a subcircuit.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Subckt {
    /// The subcircuit name.
    pub name: Substr,
    /// The list of ports.
    ///
    /// Each port is a node exposed by this subcircuit.
    pub ports: Vec<Node>,
    /// List of components in the subcircuit.
    pub components: Vec<Component>,

    /// A set of deep shorted nodes.
    ///
    /// For example, a subcircuit containing `.CONNECT node1 node2`
    /// and no other `.CONNECT` statements will yield
    /// `connects = vec![("node1", "node2")]`.
    pub connects: Vec<(Node, Node)>,
}

/// A SPICE netlist component.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Component {
    /// A MOSFET (declared with an 'M').
    Mos(Mos),
    /// A resistor (declared with an 'R').
    Res(Res),
    /// A diode (declared with a 'D').
    Diode(Diode),
    /// A capacitor (declared with a 'C').
    Cap(Cap),
    /// An instance of a subcircuit (declared with an 'X').
    Instance(Instance),
}

/// A way of specifying the value of a primitive device.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DeviceValue {
    /// The value is a fixed nominal value, e.g. `10p`.
    Value(Substr),
    /// The value is computed by a model with the given name.
    Model(Substr),
}

/// A resistor.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Res {
    /// The name of the resistor instance.
    pub name: Substr,
    /// The node connected to the positive terminal.
    pub pos: Node,
    /// The node connected to the negative terminal.
    pub neg: Node,
    /// The value or model of the resistor.
    pub value: DeviceValue,
    /// Parameters and their values.
    pub params: Params,
}

/// A diode.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Diode {
    /// The name of the diode instance.
    pub name: Substr,
    /// The node connected to the positive terminal.
    pub pos: Node,
    /// The node connected to the negative terminal.
    pub neg: Node,
    /// The name of the associated diode model.
    pub model: Substr,
    /// Parameters and their values.
    pub params: Params,
}

/// A capacitor.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Cap {
    /// The name of the capacitor instance.
    pub name: Substr,
    /// The node connected to the positive terminal.
    pub pos: Node,
    /// The node connected to the negative terminal.
    pub neg: Node,
    /// The value of the resistor.
    pub value: Substr,
}

/// A subcircuit instance.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Instance {
    /// The name of the instance.
    pub name: Substr,
    /// The list of port connections.
    pub ports: Vec<Node>,
    /// The name of the child cell.
    pub child: Substr,
    /// Instance parameters.
    pub params: Params,
}

/// A MOSFET.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Mos {
    /// The name of the MOSFET instance.
    pub name: Substr,
    /// The drain.
    pub d: Node,
    /// The gate.
    pub g: Node,
    /// The source.
    pub s: Node,
    /// The body/substrate.
    pub b: Node,
    /// The name of the associated MOSFET model.
    pub model: Substr,
    /// Parameters and their values.
    pub params: Params,
}

/// Parameter values.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Params {
    /// A map of key-value pairs.
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
fn is_whitespace_equivalent(c: char, ignore: &HashSet<char>) -> bool {
    c.is_whitespace() || ignore.contains(&c)
}

#[inline]
fn is_space_or_newline(c: char) -> bool {
    is_space(c) || is_newline(c)
}

#[inline]
fn is_special(c: char) -> bool {
    is_space_or_newline(c) || c == '='
}

struct Tokenizer {
    data: Substr,
    rem: Substr,
    state: TokState,
    comments: HashSet<char>,
    /// Characters to treat as equivalent to whitespace.
    ignore_chars: HashSet<char>,
    line_continuation: char,
    /// The string used to prefix metadata SPICE directives.
    ///
    /// In CDL format, this is "*.".
    meta_directive_prefix: Option<String>,
}

/// A SPICE token.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token {
    /// A SPICE directive that starts with a leading dot.
    ///
    /// Examples: ".subckt", ".ends", ".include".
    ///
    /// The tokenizer returns tokens with case matching the input file.
    /// No conversion to upper/lowercase is made.
    Directive(Substr),
    /// A SPICE identifier.
    Ident(Substr),
    /// A line end indicator.
    LineEnd,
    /// An equal sign token ('=').
    Equals,
    /// A metadata directive.
    ///
    /// Examples: "*.CONNECT", "*.PININFO".
    MetaDirective(Substr),
}

#[derive(Copy, Clone, Default, Eq, PartialEq, Hash, Debug)]
enum TokState {
    /// Initial state.
    #[default]
    Init,
    /// Parsing a line.
    Line,
}

/// An error arising from parsing a SPICE netlist.
#[derive(Debug, Error)]
pub enum ParserError {
    /// A tokenizer error.
    #[error("tokenizer error: {0}")]
    Tokenizer(#[from] TokenizerError),
    /// Found a SPICE line in the wrong context.
    ///
    /// For example, a ".ends" line with no matching ".subckt" line.
    #[error("unexpected line: {0:?}")]
    UnexpectedLine(Box<Line>),
    /// An unsupported or unexpected SPICE directive.
    #[error("unexpected SPICE directive: {0}")]
    UnexpectedDirective(Substr),
    /// An unsupported or unexpected SPICE component type.
    #[error("unexpected component type: {0}")]
    UnexpectedComponentType(char),
    /// An unsupported or unexpected token.
    #[error("unexpected token: {0:?}")]
    UnexpectedToken(Token),
    /// A relative path was used in an unsupported position.
    ///
    /// For example, relative paths are forbidden when parsing inline spice.
    #[error("unexpected relative path: {0:?}")]
    UnexpectedRelativePath(Substr),
    /// An invalid line.
    #[error("invalid line `{line:?}`: {reason}")]
    InvalidLine {
        /// The tokens in the offending line.
        line: Vec<Token>,
        /// The reason the line is invalid.
        reason: String,
    },
    /// Error trying to read the given file.
    #[error("failed to read file at path `{path:?}`: {err:?}")]
    FailedToRead {
        /// The path we attempted to read.
        path: PathBuf,
        /// The underlying error.
        #[source]
        err: std::io::Error,
    },
}

/// A tokenizer error.
#[derive(Debug, Error)]
#[allow(dead_code)]
pub struct TokenizerError {
    /// The state of the tokenizer at the time this error occurred.
    state: TokState,
    /// The byte offset in the file being tokenized.
    ofs: usize,
    /// The full contents of the file being parsed.
    data: Substr,
    /// The contents of `data` that have not yet been processed.
    rem: Substr,
    message: ArcStr,
    token: Substr,
}

impl Tokenizer {
    fn new(dialect: Dialect, data: impl Into<arcstr::Substr>) -> Self {
        let data = data.into();
        let rem = data.clone();
        let meta_directive_prefix = match dialect {
            Dialect::Spice => None,
            Dialect::Cdl => Some("*.".to_string()),
        };
        let ignore_chars = match dialect {
            Dialect::Spice => HashSet::new(),
            Dialect::Cdl => HashSet::from(['/']),
        };
        Self {
            data: Substr(data),
            rem: Substr(rem),
            state: TokState::Init,
            comments: HashSet::from(['*', '$']),
            ignore_chars,
            line_continuation: '+',
            meta_directive_prefix,
        }
    }

    fn next_is_meta_directive(&self) -> bool {
        self.meta_directive_prefix
            .as_ref()
            .map(|s| self.rem.starts_with(s))
            .unwrap_or_default()
    }

    fn try_meta_directive(&mut self) -> Option<Substr> {
        if self.next_is_meta_directive() {
            let s = self.meta_directive_prefix.as_ref().unwrap();
            self.rem = Substr(self.rem.substr(s.len()..));
            Some(self.take_ident())
        } else {
            None
        }
    }

    pub fn get(&mut self) -> Result<Option<Token>, TokenizerError> {
        loop {
            self.take_ws();
            if self.rem.is_empty() {
                // handle EOF
                if self.state == TokState::Line {
                    // At EOF, but have not yet returned a final LineEnd token.
                    self.state = TokState::Init;
                    return Ok(Some(Token::LineEnd));
                } else {
                    // At EOF, no more tokens.
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
                    if self.comments.contains(&c) && !self.next_is_meta_directive() {
                        self.take_until_newline();
                    } else if is_whitespace_equivalent(c, &self.ignore_chars) {
                        self.take1();
                    } else if c == self.line_continuation {
                        self.err("unexpected line continuation", c)?;
                    } else {
                        self.state = TokState::Line;
                    }
                }
                TokState::Line => {
                    if let Some(md) = self.try_meta_directive() {
                        return Ok(Some(Token::MetaDirective(md)));
                    } else if is_newline(c) {
                        self.take1();
                        self.take_ws();
                        if self.peek().unwrap_or(self.line_continuation) != self.line_continuation {
                            self.state = TokState::Init;
                            return Ok(Some(Token::LineEnd));
                        }
                    } else if c == self.line_continuation || self.ignore_chars.contains(&c) {
                        self.take1();
                    } else if self.comments.contains(&c) {
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
            rem: self.rem.clone(),
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

struct Tokens {
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

impl From<ArcStr> for Substr {
    fn from(value: ArcStr) -> Self {
        Self(arcstr::Substr::full(value))
    }
}

impl From<char> for Substr {
    fn from(value: char) -> Self {
        Self(arcstr::Substr::from(value.to_string()))
    }
}

impl Token {
    fn try_ident(&self) -> Result<&Substr, ParserError> {
        match self {
            Self::Ident(x) => Ok(x),
            _ => Err(ParserError::UnexpectedToken(self.clone())),
        }
    }
}

impl Params {
    /// Create a new, empty parameter set.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a key-value pair into the parameter set.
    pub fn insert(&mut self, k: impl Into<Substr>, v: impl Into<Substr>) {
        self.values.insert(k.into(), v.into());
    }

    /// Get the value corresponding to the given key.
    pub fn get(&self, k: &str) -> Option<&Substr> {
        self.values.get(k)
    }

    /// An iterator over all key-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&Substr, &Substr)> {
        self.values.iter()
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

impl ParsedSpice {
    /// Convert this SPICE netlist to a SCIR library.
    pub fn to_scir(&self) -> conv::ConvResult<scir::Library<Spice>> {
        let conv = ScirConverter::new(&self.ast);
        conv.convert()
    }
}
