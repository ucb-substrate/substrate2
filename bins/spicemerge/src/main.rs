use anyhow::Context;
use clap::Parser as ClapParser;
use scir::netlist::ConvertibleNetlister;
use spice::netlist::NetlistOptions;
use spice::parser::{Dialect, Parser};
use spice::Spice;
use std::io;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Some(ref out) = args.out {
        println!("input file: {:?}", &args.input);
        println!("dialect: {}", &args.dialect);
        println!("output: {:?}", &out);
        spicemerge(args)?;
        println!("Netlist writing complete.");
    } else {
        eprintln!("input file: {:?}", &args.input);
        eprintln!("dialect: {}", &args.dialect);
        eprintln!("output: stdout");
        spicemerge(args)?;
        eprintln!("Netlist writing complete.");
    }

    Ok(())
}

/// Arguments to [`spicemerge`].
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

/// Merge the given SPICE files into one netlist.
pub fn spicemerge(args: Args) -> anyhow::Result<()> {
    let parsed = Parser::parse_file(Dialect::Cdl, args.input)
        .with_context(|| "Failed to parse input file.")?;
    let lib = parsed
        .to_scir()
        .with_context(|| "Failed to convert input netlist to SCIR.")?;

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
