//! # `csv-ledger`
//!  Consumes a CSV containing a list of transactions and produces a set of bank account statements.
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
//! To see a full list of options run:
//! ```sh
//! csv-ledger --help
//! ```

pub mod bank;
pub mod parse;

use bank::{Bank, BankErr};
use clap::Parser;
use nom::Err as NomErr;
use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The path to the input CSV File.
    path: PathBuf,

    #[clap(short = 'o', long = "output")]
    /// A path to save the output a a file. By default, the output will be printed to stdout.
    output: Option<PathBuf>,
}

fn main() -> Result<(), BankErr> {
    let args = Args::parse();

    // Open the csv file
    let file = File::open(args.path)?;

    // Create a new bank and consume the csv file
    let mut bank = Bank::default();
    bank.consume_csv(BufReader::new(file))?;

    // Output the result
    if let Some(output_path) = args.output {
        fs::write(output_path, format!("{}", bank))?;
    } else {
        println!("{}", bank);
    }

    Ok(())
}
