use anyhow::Context;
use clap::Parser as ClapParser;
use spice::netlist::NetlistOptions;
use spice::parser::conv::ScirConverter;
use spice::parser::{Dialect, Parser};
use spice::Spice;
use std::io;
use std::path::PathBuf;
use substrate::arcstr::ArcStr;
use substrate::schematic::netlist::ConvertibleNetlister;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Some(ref out) = args.out {
        println!("input file: {:?}", &args.file);
        println!("blackbox cells: {:?}", &args.blackbox);
        println!("output: {:?}", &out);
        cdl2spice(args)?;
        println!("Netlist writing complete.");
    } else {
        eprintln!("input file: {:?}", &args.file);
        eprintln!("blackbox cells: {:?}", &args.blackbox);
        eprintln!("output: stdout");
        cdl2spice(args)?;
        eprintln!("Netlist writing complete.");
    }

    Ok(())
}

/// Arguments to [`cdl2spice`].
#[derive(ClapParser)]
#[command(
    version,
    about,
    long_about = "Convert a CDL netlist to a SPICE netlist"
)]
pub struct Args {
    /// The path to the input CDL netlist.
    file: PathBuf,
    /// The names of the cells to treat as blackboxes.
    #[arg(short, long)]
    blackbox: Vec<String>,
    /// The path where the output SPICE file should be saved.
    ///
    /// The file and its parent directories will be created if necessary.
    /// If the file already exists, it will be overwritten.
    ///
    /// If unspecified, the output will be written to stdout.
    #[arg(short, long)]
    out: Option<PathBuf>,
}

/// Convert the given CDL netlist to a SPICE netlist.
pub fn cdl2spice(args: Args) -> anyhow::Result<()> {
    let parsed = Parser::parse_file(Dialect::Cdl, args.file)
        .with_context(|| "Failed to parse input CDL file.")?;
    let mut converter = ScirConverter::new(&parsed.ast);
    for blackbox in args.blackbox {
        converter.blackbox(ArcStr::from(blackbox));
    }
    let lib = converter
        .convert()
        .with_context(|| "Failed to convert to SPICE.")?;
    let issues = lib.validate();
    for item in issues.iter() {
        eprintln!("{item}");
    }
    if issues.has_error() {
        anyhow::bail!("One or more errors in netlist identified; aborting.")
    }

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
