//! # `csv-ledger`
//!  Consumes a CSV containing a list of transactions and produces a set of client account statements.
//!
//! ## Installation
//!
//! ```sh
//! cargo install csv-ledger
//! ```
//!
//! ## Usage
//!
//! **Print output to console:**
//! ```sh
//! csv-ledger foo.csv
//! ```
//!
//! **Save output to file:**
//! ```sh
//! csv-ledger --output out.csv foo.csv
//! ```
//!
//! **To see help infomation:**
//! ```sh
//! csv-ledger --help
//! ```

pub mod ledger;
pub mod parse;

use clap::Parser;
use core::fmt;
use ledger::Ledger;
use nom::Err as NomErr;
use std::{
    fmt::Display,
    fs::{self, File},
    io::{self, BufReader},
    path::PathBuf,
    process::ExitCode,
};

#[derive(Debug)]
pub enum LedgerErr {
    Opening(io::Error),
    Reading(io::Error),
    Saving(io::Error),
    Parse(String),
}

impl Display for LedgerErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (msg, e) = match self {
            LedgerErr::Opening(e) => ("opening the csv", e.to_string()),
            LedgerErr::Reading(e) => ("reading in the csv", e.to_string()),
            LedgerErr::Saving(e) => ("saving the output file", e.to_string()),
            LedgerErr::Parse(e) => ("parsing csv", e.clone()),
        };

        write!(f, "Ledger Error 🦀 - Issue whilst {msg}: {e}",)
    }
}

impl<E> From<NomErr<E>> for LedgerErr
where
    E: Display,
{
    fn from(err: NomErr<E>) -> Self {
        LedgerErr::Parse(match err {
            NomErr::Incomplete(_) => "Input was incomplete.".to_string(),
            NomErr::Error(e) => format!("Input was in the wrong format. Error: {e}"),
            NomErr::Failure(_) => "Faliure whilst parsing input.".to_string(),
        })
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// The path to the input CSV File.
    path: PathBuf,

    #[clap(short = 'o', long = "output")]
    /// A path to save the output a a file. By default, the output will be printed to stdout.
    output: Option<PathBuf>,
}

fn main() -> ExitCode {
    let args = Args::parse();

    if let Err(err) = perform_parse_and_output(args.path, args.output) {
        println!("{err}");
        return ExitCode::from(1);
    }

    ExitCode::from(0)
}

fn perform_parse_and_output(path: PathBuf, output: Option<PathBuf>) -> Result<(), LedgerErr> {
    // Open the csv file
    let file = File::open(path).map_err(LedgerErr::Opening)?;

    // Create a new ledger and consume the csv file
    let mut ledger = Ledger::default();
    ledger.consume_csv(BufReader::new(file))?;

    // Output the result
    if let Some(output_path) = output {
        fs::write(output_path, format!("{ledger}")).map_err(LedgerErr::Saving)?;
    } else {
        println!("{}", ledger);
    }

    Ok(())
}
