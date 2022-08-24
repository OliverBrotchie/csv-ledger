//! # Zero-coppy CSV Parsing
//!  Validate headers and parse transactions from text.
//!
//! **Basic example:**
//! ```rust
//! use crate::parse::{parse_header, parse_transaction}
//!
//! fn main() {
//!     // Example csv data
//!     let csv = "type, client, tx, amount,
//!         deposit, 1, 1, 17.99
//!         withdrawal, 2, 2, 12.00
//!         hold 1, 1, ";
//!     
//!     let mut lines = csv.split("\n");
//!     let mut transactions = Vec::new();
//!     
//!     // Validate that the header is in the correct format
//!     parse_header(lines.next().unwrap()).expect("Header was invalid.");
//!     
//!     // Insert all transactions into a vector
//!     for line in lines {
//!         transactions.push(parse_transaction(line).expect("Transaction was invalid."));
//!     }
//!     
//!     // Print out the vector
//!     println!("{:?}", transactions);
//! }
//! ```

extern crate nom;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{multispace0, u16, u32},
    combinator::opt,
    error::{Error as SubErr, ErrorKind, ParseError},
    number::complete::double,
    sequence::{delimited, terminated},
    Err as NomErr, IResult,
};

/// An enum that represents possible transaction types.
#[derive(Debug, PartialEq)]
pub enum Transaction {
    Deposit(u16, u32, f64),
    Withdrawal(u16, u32, f64),
    Dispute(u16, u32),
    Resolve(u16, u32),
    Chargeback(u16, u32),
}

/// A parser that ignores whitespace around the input.
fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

/// Parse the CSV header to validate that the CSV is in the correct format.
/// Please note that whitespace will be ignored.
///
/// Example:
/// ```rs
/// assert!(parse_header("type, client, tx, amount").is_ok());
/// assert!(parse_header(" type,  client, tx  ,amount  ").is_ok());
///
/// assert!(parse_header("type, client, tx").is_err());
/// ```
pub fn parse_header(input: &str) -> IResult<&str, ()> {
    let (input, _) = terminated(ws(tag("type")), tag(","))(input)?;
    let (input, _) = terminated(ws(tag("client")), tag(","))(input)?;
    let (input, _) = terminated(ws(tag("tx")), tag(","))(input)?;
    let (input, _) = ws(tag("amount"))(input)?;

    if !input.is_empty() {
        return Err(NomErr::Failure(SubErr {
            input,
            code: ErrorKind::Fail,
        }));
    }

    Ok((input, ()))
}

/// Parse a line of the CSV as a Transaction.
/// Please note that whitespace will be ignored.
///
/// Example:
/// ```ts
/// // Valid Inputs:
/// assert_eq!(parse_transaction("deposit, 1, 1, 20.0"), ("", Transaction::Deposit(1, 1, 20.0)))
/// assert_eq!(parse_transaction(" deposit,  2, 20  ,6.99  "), ("", Transaction::Deposit(2, 20, 6.0)));
/// assert_eq!(parse_transaction("withdrawal, 3, 7, 22.7"), ("", Transaction::Withdrawal(3, 7, 22.0)));
///
/// assert_eq!(parse_transaction("dispute, 2, 2,"), ("", Transaction::Dispute(2, 2)));
/// assert_eq!(parse_transaction("resolve, 2, 2,"), ("", Transaction::Resolve(2, 2)));
///
/// assert_eq!(parse_transaction("dispute, 3, 7,"), ("", Transaction::Dispute(3, 7)));
/// assert_eq!(parse_transaction("chargeback, 3, 7,"), ("", Transaction::Chargeback(3, 7)));
///
/// // Invalid Inputs:
/// assert!(parse_transaction("deposit, 1, 1,").is_err());
/// assert!(parse_transaction("xyz, 1, 1, 2.0").is_err());
/// assert!(parse_transaction("dispute, 1,").is_err());
/// ```
pub fn parse_transaction(input: &str) -> IResult<&str, Transaction> {
    // Parse the type of Transaction
    let (input, key) = terminated(
        ws(alt((
            tag("deposit"),
            tag("withdrawal"),
            tag("dispute"),
            tag("resolve"),
            tag("chargeback"),
        ))),
        tag(","),
    )(input)?;

    // Parse the account and Transaction ID
    let (input, client) = terminated(ws(u16), tag(","))(input)?;
    let (input, tx) = terminated(ws(u32), tag(","))(input)?;

    // Parse the Transaction amount
    let (input, amount) = opt(ws(double))(input)?;

    if !input.is_empty() {
        return Err(NomErr::Failure(SubErr {
            input,
            code: ErrorKind::Fail,
        }));
    }

    // Amounts must be positive
    if let Some(value) = amount && value.is_sign_negative() {
        return Err(NomErr::Failure(SubErr {
            input: "Amount was negative.",
            code: ErrorKind::Fail,
        }));
    }

    // Convert values into Transaction
    Ok((
        input,
        match (key, amount) {
            ("deposit", Some(value)) => Transaction::Deposit(client, tx, value),
            ("withdrawal", Some(value)) => Transaction::Withdrawal(client, tx, value),
            ("dispute", None) => Transaction::Dispute(client, tx),
            ("resolve", None) => Transaction::Resolve(client, tx),
            ("chargeback", None) => Transaction::Chargeback(client, tx),
            (_, _) => Err(NomErr::Failure(SubErr {
                input: if key == "deposit" || key == "withdrawal" {
                    "Deposit or Withdrawal without an amount."
                } else {
                    "Dispute, Resolve or Chargeback with an amount."
                },
                code: ErrorKind::NonEmpty,
            }))?,
        },
    ))
}

#[cfg(test)]
mod parse_header {
    use crate::parse::parse_header;

    #[test]
    fn ok_no_white_space() {
        parse_header("type,client,tx,amount").expect("Error whilst parsing header.");
    }

    #[test]
    fn ok_with_white_space() {
        parse_header("   type    ,  client,   tx  ,    amount    ")
            .expect("Error whilst parsing header.");
    }

    #[test]
    fn err_invalid_input() {
        parse_header("client,type,ammount,tx").unwrap_err();
    }

    #[test]
    fn err_missing_value() {
        parse_header("type,client,tx,").unwrap_err();
    }

    #[test]
    fn err_extra_value() {
        parse_header("type,client,tx,amount,foo").unwrap_err();
    }
}

#[cfg(test)]
mod parse_transaction {
    use crate::parse::{parse_transaction, Transaction};

    #[test]
    fn deposit() {
        let (_, res) = parse_transaction("deposit, 1, 2, 3.0").unwrap();
        assert_eq!(res, Transaction::Deposit(1, 2, 3.0));
    }

    #[test]
    fn withdrawal() {
        let (_, res) = parse_transaction("withdrawal, 1, 2, 3.0").unwrap();
        assert_eq!(res, Transaction::Withdrawal(1, 2, 3.0));
    }

    #[test]
    fn dispute() {
        let (_, res) = parse_transaction("dispute, 1, 2,").unwrap();
        assert_eq!(res, Transaction::Dispute(1, 2));
    }

    #[test]
    fn resolve() {
        let (_, res) = parse_transaction("resolve, 1, 2,").unwrap();
        assert_eq!(res, Transaction::Resolve(1, 2));
    }

    #[test]
    fn chargeback() {
        let (_, res) = parse_transaction("chargeback, 1, 2,").unwrap();
        assert_eq!(res, Transaction::Chargeback(1, 2));
    }

    #[test]
    fn ok_no_white_space() {
        let (_, res) = parse_transaction("deposit,1,2,3.0").unwrap();

        assert_eq!(res, Transaction::Deposit(1, 2, 3.0));
    }

    #[test]
    fn ok_with_white_space() {
        let (_, res) = parse_transaction("   deposit   ,1  ,   2,  3.0  ").unwrap();
        assert_eq!(res, Transaction::Deposit(1, 2, 3.0));
    }

    #[test]
    fn ok_no_amount() {
        let (_, res) = parse_transaction("dispute,1,2,").unwrap();
        assert_eq!(res, Transaction::Dispute(1, 2));
    }

    #[test]
    fn err_invalid_u16() {
        parse_transaction("deposit,65536,2,3.0").unwrap_err();
    }

    #[test]
    fn err_invalid_deposit() {
        parse_transaction("deposit,1,2,").unwrap_err();
    }

    #[test]
    fn err_dispute_missing_value() {
        parse_transaction("dispute,1,").unwrap_err();
    }

    #[test]
    fn err_dispute_extra_value() {
        parse_transaction("dispute,1,2,3.0").unwrap_err();
    }

    #[test]
    fn err_extra_value() {
        parse_transaction("withdrawal,1,2,3.0,foo").unwrap_err();
    }

    #[test]
    fn err_negative_amount() {
        parse_transaction("withdrawal,1,2,-3.0").unwrap_err();
    }
}
