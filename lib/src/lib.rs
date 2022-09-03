//! # `csv_ledger_lib`
//!  A sub-library for the `csv_leger` CLI. This library contains two modules:
//! - `ledger`: Containing the `Ledger` state store.
//! - `parse`: Containing a zero-coppy csv parser.

pub mod ledger;
pub mod parse;

use core::fmt;
use nom::Err as NomErr;
use std::{fmt::Display, io};

#[derive(Debug)]
pub enum LedgerErr {
    Opening(io::Error),
    Reading(io::Error),
    Saving(io::Error),
    Parse(String, usize),
}

impl LedgerErr {
    fn from_parse<E>(err: NomErr<E>, index: usize) -> LedgerErr {
        LedgerErr::Parse(
            match err {
                NomErr::Incomplete(_) => "Input was incomplete",
                NomErr::Error(_) => "Input was in the wrong format",
                NomErr::Failure(_) => "Faliure whilst parsing input",
            }
            .to_string(),
            index,
        )
    }
}

impl Display for LedgerErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (msg, e) = match self {
            LedgerErr::Opening(e) => ("opening the csv", e),
            LedgerErr::Reading(e) => ("reading in the csv", e),
            LedgerErr::Saving(e) => ("saving the output file", e),
            LedgerErr::Parse(e, index) => {
                return write!(
                    f,
                    "Ledger Error 🦀 - Issue whilst parsing csv: \"{}\", At line: {index}",
                    e
                )
            }
        };

        write!(f, "Ledger Error 🦀 - Issue whilst {msg}: {}", e)
    }
}

#[cfg(test)]
mod ledger_err {
    use crate::LedgerErr;
    use nom::{error::ErrorKind, Err as NomErr, Needed};

    #[test]
    fn from_parse() {
        assert_eq!(
            LedgerErr::from_parse(NomErr::Incomplete::<Needed>(Needed::Unknown), 1).to_string(),
            "Ledger Error 🦀 - Issue whilst parsing csv: \"Input was incomplete\", At line: 1",
        );

        assert_eq!(
            LedgerErr::from_parse(NomErr::Failure(("ERROR", ErrorKind::Fail)), 1).to_string(),
            "Ledger Error 🦀 - Issue whilst parsing csv: \"Faliure whilst parsing input\", At line: 1",
        );

        assert_eq!(
            LedgerErr::from_parse(NomErr::Error(("ERROR", ErrorKind::Fail)), 1).to_string(),
            "Ledger Error 🦀 - Issue whilst parsing csv: \"Input was in the wrong format\", At line: 1",
        );
    }

    #[test]
    fn debug() {
        let err = super::LedgerErr::Opening(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found",
        ));
        assert_eq!(
            format!("{:?}", err),
            "Opening(Custom { kind: NotFound, error: \"File not found\" })",
        );
    }

    #[test]
    fn display() {
        assert_eq!(
            format!(
                "{}",
                super::LedgerErr::Opening(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "File not found",
                ))
            ),
            "Ledger Error 🦀 - Issue whilst opening the csv: File not found",
        );

        assert_eq!(
            format!(
                "{}",
                super::LedgerErr::Reading(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "File not found",
                ))
            ),
            "Ledger Error 🦀 - Issue whilst reading in the csv: File not found",
        );

        assert_eq!(
            format!(
                "{}",
                super::LedgerErr::Saving(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "File not found",
                ))
            ),
            "Ledger Error 🦀 - Issue whilst saving the output file: File not found",
        );

        assert_eq!(
            format!("{}", super::LedgerErr::Parse("ERROR".into(), 1)),
            "Ledger Error 🦀 - Issue whilst parsing csv: \"ERROR\", At line: 1"
        );
    }
}
