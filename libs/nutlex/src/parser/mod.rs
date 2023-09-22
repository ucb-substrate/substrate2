//! The rawfile parser and AST data structures.
use std::str;

use nom::branch::alt;
use nom::bytes::complete::{tag_no_case, take, take_till1, take_while, take_while1};
use nom::character::complete::{line_ending, space0, space1};
use nom::combinator::opt;
use nom::error::{Error, ErrorKind};
use nom::multi::many0;
use nom::number::complete::{be_f64, le_f64};
use nom::sequence::{delimited, tuple};
use nom::{Err, IResult};
use serde::{Deserialize, Serialize};

use crate::{ByteOrder, Options};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Data stored by a single analysis.
pub struct Analysis<'a> {
    /// The title of the analysis.
    pub title: Option<&'a str>,
    /// The date on which the analysis was performed.
    pub date: Option<&'a str>,
    /// Plot name.
    pub plotname: &'a str,
    /// Flags.
    pub flags: &'a str,
    /// The number of saved variables.
    pub num_variables: usize,
    /// The number of points saved.
    pub num_points: usize,
    /// The saved variable names.
    pub variables: Vec<Variable<'a>>,
    /// The saved variable values.
    pub data: AnalysisData,
}

/// Data saved by an [`Analysis`].
pub type AnalysisData = Data<Vec<RealSignal>, Vec<ComplexSignal>>;

/// Data saved by an [`Analysis`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[enumify::enumify]
pub enum Data<R, C> {
    /// A set of real signals.
    Real(R),
    /// A set of complex signals.
    Complex(C),
}

/// A real data vector.
pub type RealSignal = Vec<f64>;

/// A complex data vector.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplexSignal {
    /// The real part.
    pub real: Vec<f64>,
    /// The imaginary part.
    pub imag: Vec<f64>,
}

impl ComplexSignal {
    fn with_capacity(cap: usize) -> Self {
        Self {
            real: Vec::with_capacity(cap),
            imag: Vec::with_capacity(cap),
        }
    }
}

/// A variable saved in an [`Analysis`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variable<'a> {
    /// The index of this variable in the list of saved data vectors.
    pub idx: usize,
    /// The name of the signal.
    pub name: &'a str,
    /// The signal units.
    pub unit: &'a str,
}

fn is_newline(c: u8) -> bool {
    c == b'\n' || c == b'\r'
}

fn is_space_or_line(c: u8) -> bool {
    c == b'\n' || c == b'\r' || c == b' ' || c == b'\t'
}

fn header<'a, 'b>(key: &'b str) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], &'a str> + 'b {
    move |input| {
        let tag = tag_no_case(key);
        let (input, _) = space0(input)?;
        let header_value = take_till1(is_newline);
        let (input, value) = delimited(tag, header_value, line_ending)(input)?;
        let value = from_utf8(value)?;
        Ok((input, value))
    }
}

fn from_utf8(input: &[u8]) -> Result<&str, Err<Error<&[u8]>>> {
    let value = std::str::from_utf8(input)
        .map_err(|_| Err::Failure(Error::new(input, ErrorKind::Permutation)))?;
    Ok(value)
}

fn parse_usize(input: &[u8]) -> Result<usize, Err<Error<&[u8]>>> {
    let string = from_utf8(input)?;
    let value = string
        .trim()
        .parse::<usize>()
        .map_err(|_| Err::Failure(Error::new(input, ErrorKind::Fail)))?;
    Ok(value)
}

fn parse_usize_str(input: &str) -> Result<usize, Err<Error<&[u8]>>> {
    let value = input
        .trim()
        .parse::<usize>()
        .map_err(|_| Err::Failure(Error::new(input.as_bytes(), ErrorKind::TagClosure)))?;
    Ok(value)
}

fn parse_f64(input: &[u8]) -> Result<f64, Err<Error<&[u8]>>> {
    let string = from_utf8(input)?;
    let value = string
        .trim()
        .parse::<f64>()
        .map_err(|_| Err::Failure(Error::new(input, ErrorKind::TagClosure)))?;
    Ok(value)
}

fn variable(input: &[u8]) -> IResult<&[u8], Variable> {
    let value = take_till1(is_space_or_line);
    // In AC analysis, may have a `grid=X` declaration
    let kwargs = opt(take_till1(is_newline));
    let (input, (_, idx, _, name, _, unit, _, _, _, _)) = tuple((
        space0,
        &value,
        space1,
        &value,
        space1,
        &value,
        space0,
        kwargs,
        space0,
        line_ending,
    ))(input)?;
    let idx = parse_usize(idx)?;
    let name = from_utf8(name)?;
    let unit = from_utf8(unit)?;
    Ok((input, Variable { idx, name, unit }))
}

fn variables(input: &[u8]) -> IResult<&[u8], Vec<Variable>> {
    let (input, _) = tuple((tag_no_case("Variables:"), space0, opt(line_ending), space0))(input)?;
    let (input, vars) = many0(variable)(input)?;
    Ok((input, vars))
}

fn real_data_binary(
    vars: usize,
    points: usize,
    opts: Options,
) -> impl Fn(&[u8]) -> IResult<&[u8], AnalysisData> {
    move |input| {
        let (mut input, _) = tuple((tag_no_case("Binary:"), space0, line_ending))(input)?;
        let mut out = vec![Vec::with_capacity(points); vars];
        for _ in 0..points {
            for item in out.iter_mut().take(vars) {
                let val: f64;
                (input, val) = match opts.endianness {
                    ByteOrder::BigEndian => be_f64(input)?,
                    ByteOrder::LittleEndian => le_f64(input)?,
                };
                item.push(val);
            }
        }

        Ok((input, AnalysisData::Real(out)))
    }
}

fn real_data_ascii(vars: usize, points: usize) -> impl Fn(&[u8]) -> IResult<&[u8], AnalysisData> {
    move |input| {
        let (mut input, _) = tuple((tag_no_case("Values:"), space0, line_ending))(input)?;
        (input, _) = take_while(is_space_or_line)(input)?;

        let mut out = vec![Vec::with_capacity(points); vars];
        for _ in 0..points {
            (input, _) = take_till1(is_space_or_line)(input)?;
            for item in out.iter_mut().take(vars) {
                let val;
                (input, _) = take_while1(is_space_or_line)(input)?;
                (input, val) = take_till1(is_space_or_line)(input)?;
                item.push(parse_f64(val)?);
            }
            (input, _) = take_while1(is_space_or_line)(input)?;
        }

        Ok((input, AnalysisData::Real(out)))
    }
}

fn real_data(
    input: &[u8],
    vars: usize,
    points: usize,
    opts: Options,
) -> IResult<&[u8], AnalysisData> {
    alt((
        real_data_binary(vars, points, opts),
        real_data_ascii(vars, points),
    ))(input)
}

fn complex_data_binary(
    vars: usize,
    points: usize,
    opts: Options,
) -> impl Fn(&[u8]) -> IResult<&[u8], AnalysisData> {
    move |input| {
        let (mut input, _) = tuple((tag_no_case("Binary:"), space0, line_ending))(input)?;

        let mut out = vec![ComplexSignal::with_capacity(points); vars];
        for _ in 0..points {
            for item in out.iter_mut().take(vars) {
                let val: f64;
                (input, val) = match opts.endianness {
                    ByteOrder::BigEndian => be_f64(input)?,
                    ByteOrder::LittleEndian => le_f64(input)?,
                };
                item.real.push(val);
                let val: f64;
                (input, val) = match opts.endianness {
                    ByteOrder::BigEndian => be_f64(input)?,
                    ByteOrder::LittleEndian => le_f64(input)?,
                };
                item.imag.push(val);
            }
        }

        Ok((input, AnalysisData::Complex(out)))
    }
}

fn complex_data_ascii(
    vars: usize,
    points: usize,
) -> impl Fn(&[u8]) -> IResult<&[u8], AnalysisData> {
    move |input| {
        let (mut input, _) = tuple((tag_no_case("Values:"), space0, line_ending))(input)?;
        (input, _) = take_while(is_space_or_line)(input)?;

        let mut out = vec![ComplexSignal::with_capacity(points); vars];
        for _ in 0..points {
            (input, _) = take_till1(is_space_or_line)(input)?;
            for item in out.iter_mut().take(vars) {
                (input, _) = take_while1(is_space_or_line)(input)?;
                let val;
                (input, val) = take_till1(|c| c == b',')(input)?;
                item.real.push(parse_f64(val)?);
                (input, _) = take(1u64)(input)?;
                let val;
                (input, val) = take_till1(is_space_or_line)(input)?;
                item.imag.push(parse_f64(val)?);
            }
            (input, _) = take_while1(is_space_or_line)(input)?;
        }

        Ok((input, AnalysisData::Complex(out)))
    }
}

fn complex_data(
    input: &[u8],
    vars: usize,
    points: usize,
    opts: Options,
) -> IResult<&[u8], AnalysisData> {
    alt((
        complex_data_binary(vars, points, opts),
        complex_data_ascii(vars, points),
    ))(input)
}

fn analysis(opts: Options) -> impl Fn(&[u8]) -> IResult<&[u8], Analysis> {
    move |input: &[u8]| -> IResult<&[u8], Analysis> {
        let (input, _) = take_while(is_space_or_line)(input)?;
        let (input, title) = opt(header("Title:"))(input)?;
        let (input, date) = opt(header("Date:"))(input)?;
        let (input, plotname) = header("Plotname:")(input)?;
        let (input, flags) = header("Flags:")(input)?;
        let (input, num_variables) = header("No. Variables:")(input)?;
        let num_variables = parse_usize_str(num_variables)?;
        let (input, num_points) = header("No. Points:")(input)?;
        let num_points = parse_usize_str(num_points)?;
        let (input, variables) = variables(input)?;

        let (input, data) = if flags.contains("complex") {
            complex_data(input, num_variables, num_points, opts)?
        } else {
            real_data(input, num_variables, num_points, opts)?
        };

        Ok((
            input,
            Analysis {
                title,
                date,
                plotname,
                flags,
                num_variables,
                num_points,
                variables,
                data,
            },
        ))
    }
}

pub(crate) fn analyses(input: &[u8], opts: Options) -> IResult<&[u8], Vec<Analysis>> {
    many0(analysis(opts))(input)
}
