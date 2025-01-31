use anyhow::Context;
use clap::Parser as ClapParser;
use scir::netlist::ConvertibleNetlister;
use sky130::{Sky130, Sky130CdsSchema, Sky130OpenSchema, Sky130SrcNdaSchema};
use spice::netlist::NetlistOptions;
use spice::parser::{Dialect, Parser};
use spice::Spice;
use std::fmt::Display;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Some(ref out) = args.out {
        println!("input file: {:?}", &args.input);
        println!("dialect: {}", &args.dialect);
        println!("schema: {}", &args.schema);
        println!("output: {:?}", &out);
        spconv(args)?;
        println!("Netlist writing complete.");
    } else {
        eprintln!("input file: {:?}", &args.input);
        eprintln!("dialect: {}", &args.dialect);
        eprintln!("schema: {}", &args.schema);
        eprintln!("output: stdout");
        spconv(args)?;
        eprintln!("Netlist writing complete.");
    }

    Ok(())
}

/// Arguments to [`spconv`].
#[derive(ClapParser)]
#[command(
    version,
    about,
    long_about = "Aggregate a SPICE netlist (with potentially many include statements) into one file"
)]
pub struct Args {
    /// The SPICE dialect.
    #[arg(short, long, default_value_t)]
    dialect: Dialect,

    /// The desired output SKY130 schema (open, cds, or src-nda).
    #[arg(short, long, default_value_t)]
    schema: Sky130Schema,

    /// The path where the output SPICE file should be saved.
    ///
    /// The file and its parent directories will be created if necessary.
    /// If the file already exists, it will be overwritten.
    ///
    /// If unspecified, the output will be written to stdout.
    #[arg(short, long)]
    out: Option<PathBuf>,

    /// The input netlist file.
    input: PathBuf,
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum Sky130Schema {
    /// Open-source sky130 schema.
    #[default]
    Open,
    /// Cadence sky130 schema.
    Cds,
    /// NDA sky130 schema.
    SrcNda,
}

impl Display for Sky130Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Cds => write!(f, "cds"),
            Self::SrcNda => write!(f, "src-nda"),
        }
    }
}

/// Error parsing sky130 schema.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
#[error("error parsing sky130 schema")]
pub struct Sky130SchemaParseErr;

impl FromStr for Sky130Schema {
    type Err = Sky130SchemaParseErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open" => Ok(Self::Open),
            "cds" => Ok(Self::Cds),
            "src-nda" => Ok(Self::SrcNda),
            _ => Err(Sky130SchemaParseErr),
        }
    }
}

/// Merge the given SPICE file into one netlist, converting to the desired schema.
pub fn spconv(args: Args) -> anyhow::Result<()> {
    let parsed = Parser::parse_file(args.dialect, args.input)
        .with_context(|| "Failed to parse input file.")?;
    let lib = parsed
        .to_scir()
        .with_context(|| "Failed to convert input netlist to SCIR.")?;
    let lib = lib.convert_schema::<Sky130>().unwrap();
    let lib = lib.build().unwrap();
    let lib = match args.schema {
        Sky130Schema::Open => lib
            .convert_schema::<Sky130OpenSchema>()?
            .build()?
            .convert_schema::<Spice>()?
            .build()?,
        Sky130Schema::Cds => lib
            .convert_schema::<Sky130CdsSchema>()?
            .build()?
            .convert_schema::<Spice>()?
            .build()?,
        Sky130Schema::SrcNda => lib
            .convert_schema::<Sky130SrcNdaSchema>()?
            .build()?
            .convert_schema::<Spice>()?
            .build()?,
    };

    if let Some(path) = args.out {
        Spice
            .write_scir_netlist_to_file(&lib, &path, NetlistOptions::default())
            .with_context(|| format!("Failed to export SPICE netlist to {:?}.", path))?;
    } else {
        let mut stdout = io::stdout().lock();
        Spice
            .write_scir_netlist(&lib, &mut stdout, NetlistOptions::default())
            .with_context(|| "Failed to export SPICE netlist to stdout.")?;
    }

    Ok(())
}
