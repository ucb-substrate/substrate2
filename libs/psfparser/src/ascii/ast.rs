use num::complex::Complex64;

#[derive(Debug, Clone, PartialEq)]
pub struct PsfAst<'a> {
    pub header: Header<'a>,
    pub types: Vec<TypeDef<'a>>,
    pub sweeps: Vec<Sweep<'a>>,
    pub traces: Vec<Trace<'a>>,
    pub values: Vec<SignalValues<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Header<'a> {
    pub values: Vec<NamedValue<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef<'a> {
    pub name: &'a str,
    pub kinds: Vec<Kind<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamedValue<'a> {
    pub name: &'a str,
    pub value: Value<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    Int(i64),
    Real(f64),
    Str(&'a str),
    NaN,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sweep<'a> {
    pub name: &'a str,
    pub sweep_type: &'a str,
    pub kinds: Vec<Kind<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Kind<'a> {
    Float,
    Double,
    Complex,
    Int,
    Byte,
    Long,
    String,
    Array,
    Struct(Vec<TypeDef<'a>>),
    Prop(Prop<'a>),
    Star,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Trace<'a> {
    Group { name: &'a str, count: i64 },
    Signal { name: &'a str, units: &'a str },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Prop<'a> {
    pub values: Vec<NamedValue<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SignalValues<'a> {
    pub signal: &'a str,
    pub sigtype: Option<&'a str>,
    pub values: Values,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Values {
    Complex(Vec<Complex64>),
    Real(Vec<f64>),
}
