mod bank;
mod parse;

use bank::{Bank, BankErr};
use nom::Err as NomErr;
use std::{fs::File, io::BufReader};

fn main() -> Result<(), BankErr> {
    let file = File::open("./data/1.csv")?;
    let reader = BufReader::new(file);

    let mut bank = Bank::default();
    bank.consume_csv(reader)?;

    println!("{}", bank);

    Ok(())
}
