mod bank;
mod parse;

use bank::{Bank, BankErr};
use clap::Parser;
use nom::Err as NomErr;
use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

/// Consumes a CSV containing a list of transactions and produces a list of bank account statements.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The path to the input CSV File.
    #[clap(last = true)]
    path: PathBuf,

    #[clap(short = 'o', long = "output")]
    /// A path to save the output. By default, the output will be printed to stdout.
    output: Option<PathBuf>,
}

fn main() -> Result<(), BankErr> {
    let args = Args::parse();

    let file = File::open(args.path)?;
    let reader = BufReader::new(file);

    let mut bank = Bank::default();
    bank.consume_csv(reader)?;

    if let Some(output_path) = args.output {
        fs::write(output_path, format!("{}", bank))?;
    } else {
        println!("{}", bank);
    }

    Ok(())
}
