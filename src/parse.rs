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
    bytes::complete::{tag, take_while, take_while_m_n},
    character::{
        complete::{multispace0, u16, u32},
        is_digit,
    },
    error::{Error as SubErr, ErrorKind, ParseError},
    sequence::{delimited, terminated},
    Err as NomErr, IResult,
};

/// An enum that represents possible transaction types.
#[derive(Debug, PartialEq, Eq)]
pub enum Transaction {
    Deposit(u16, u32, i64),
    Withdrawal(u16, u32, i64),
    Dispute(u16, u32),
    Resolve(u16, u32),
    Chargeback(u16, u32),
}

/// A helper function to construct nom errors from custom strings.
pub fn nom_err(input: &str) -> NomErr<SubErr<&str>> {
    NomErr::Failure(SubErr {
        input,
        code: ErrorKind::Fail,
    })
}

/// A parser that ignores whitespace around the input parser.
fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

/// Test if a character is a digit.
pub fn digit(chr: char) -> bool {
    chr.is_ascii() && is_digit(chr as u8)
}

/// Parse a i64 number from a string, optionally allowing a maximum number of digits to be specified.
pub fn double(input: &str, max: Option<usize>) -> IResult<&str, i64> {
    let (input, num) = match max {
        Some(m) => take_while_m_n(1, m, digit)(input),
        None => take_while(digit)(input),
    }?;

    // Convert the string to i64
    Ok((
        input,
        num.parse::<i64>()
            .map_err(|_| nom_err("Could not parse number as i64."))?,
    ))
}

#[inline]
/// Parse an up to four decimal place number as an i64 by multiplying by 10000.
pub fn four_dp(input: &str) -> IResult<&str, i64> {
    let (input, pre_dp) = double(input, None)?;

    // Optionally parse decimal places
    if let Ok((input, _)) = tag::<_, _, (&str, ErrorKind)>(".")(input) {
        let (input, post_dp) = double(input, Some(4))?;

        // Convert decimal places to whole numbers
        return Ok((
            input,
            (pre_dp * 10000 + post_dp * 10_i64.pow(3 - post_dp.checked_ilog10().unwrap_or(0))),
        ));
    }

    Ok((input, (pre_dp * 10000)))
}

/// Parse a line of the CSV as a Transaction.
/// Please note that whitespace will be ignored.
///
/// Example:
/// ```ts
/// // Valid Inputs:
/// assert_eq!(parse_transaction("deposit, 1, 1, 20.0"), ("", Transaction::Deposit(1, 1, 200)))
/// assert_eq!(parse_transaction(" deposit,  2, 20  ,6.99  "), ("", Transaction::Deposit(2, 20, 699)));
/// assert_eq!(parse_transaction("withdrawal, 3, 7, 22.7"), ("", Transaction::Withdrawal(3, 7, 2270)));
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
#[inline]
pub fn parse_transaction(input: &str) -> Result<Transaction, NomErr<SubErr<&str>>> {
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
    let amount = delimited(multispace0, four_dp, multispace0)(input).ok();

    // Check that the line has been consumed completely
    if let Some((input,_)) =  amount && !input.is_empty() {
        Err(nom_err("Input was not empty after parsing transaction."))?;
    }

    // Convert result into Transaction
    Ok(match (key, amount) {
        ("deposit", Some((_, value))) => Transaction::Deposit(client, tx, value),
        ("withdrawal", Some((_, value))) => Transaction::Withdrawal(client, tx, value),
        ("dispute", None) => Transaction::Dispute(client, tx),
        ("resolve", None) => Transaction::Resolve(client, tx),
        ("chargeback", None) => Transaction::Chargeback(client, tx),
        (_, _) => Err(nom_err(if key == "deposit" || key == "withdrawal" {
            "Deposit or Withdrawal with a missing or invalid amount."
        } else {
            "Dispute, Resolve or Chargeback with an amount."
        }))?,
    })
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
#[inline]
pub fn parse_header(input: &str) -> Result<(), NomErr<SubErr<&str>>> {
    let (input, _) = terminated(ws(tag("type")), tag(","))(input)?;
    let (input, _) = terminated(ws(tag("client")), tag(","))(input)?;
    let (input, _) = terminated(ws(tag("tx")), tag(","))(input)?;
    let (input, _) = ws(tag("amount"))(input)?;

    if !input.is_empty() {
        return Err(nom_err("Input was not empty after parsing transaction."));
    }

    Ok(())
}

#[cfg(test)]
mod parse_transaction {
    use crate::parse::{parse_transaction, Transaction};

    #[test]
    fn deposit() {
        let res = parse_transaction("deposit, 1, 2, 3.1").unwrap();
        assert_eq!(res, Transaction::Deposit(1, 2, 31000));
    }

    #[test]
    fn withdrawal() {
        let res = parse_transaction("withdrawal, 1, 2, 3.0").unwrap();
        assert_eq!(res, Transaction::Withdrawal(1, 2, 30000));
    }

    #[test]
    fn dispute() {
        let res = parse_transaction("dispute, 1, 2,").unwrap();
        assert_eq!(res, Transaction::Dispute(1, 2));
    }

    #[test]
    fn resolve() {
        let res = parse_transaction("resolve, 1, 2,").unwrap();
        assert_eq!(res, Transaction::Resolve(1, 2));
    }

    #[test]
    fn chargeback() {
        let res = parse_transaction("chargeback, 1, 2,").unwrap();
        assert_eq!(res, Transaction::Chargeback(1, 2));
    }

    #[test]
    fn ok_no_white_space() {
        let res = parse_transaction("deposit,1,2,3.0").unwrap();

        assert_eq!(res, Transaction::Deposit(1, 2, 30000));
    }

    #[test]
    fn ok_with_white_space() {
        let res = parse_transaction("       deposit   ,1  ,   2,  3.0  ").unwrap();
        assert_eq!(res, Transaction::Deposit(1, 2, 30000));
    }

    #[test]
    fn ok_no_amount() {
        let res = parse_transaction("dispute,1,2,").unwrap();
        assert_eq!(res, Transaction::Dispute(1, 2));
    }

    #[test]
    fn err_parser_runthrough() {
        parse_transaction("x").unwrap_err();
        parse_transaction("deposit,x").unwrap_err();
        parse_transaction("deposit,1,x").unwrap_err();
        parse_transaction("deposit,1,2,x").unwrap_err();
        parse_transaction(&format!("deposit,1,2,2{}", f32::MAX)).unwrap_err();
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
    fn err_withdrawal_missing_value() {
        let res = parse_transaction("withdrawal,1,2,").unwrap_err();
        assert_eq!(
            res.to_string(),
            "Parsing Failure: Error { input: \"Deposit or Withdrawal with a missing or invalid amount.\", code: Fail }"
        );
    }

    #[test]
    fn err_deposit_missing_value() {
        let res = parse_transaction("deposit,1,2,").unwrap_err();
        assert_eq!(
            res.to_string(),
            "Parsing Failure: Error { input: \"Deposit or Withdrawal with a missing or invalid amount.\", code: Fail }"
        );
    }

    #[test]
    fn err_dispute_extra_value() {
        let res = parse_transaction("dispute,1,2,3.0").unwrap_err();

        assert_eq!(
            res.to_string(),
            "Parsing Failure: Error { input: \"Dispute, Resolve or Chargeback with an amount.\", code: Fail }"
        );
    }

    #[test]
    fn err_extra_value() {
        parse_transaction("withdrawal,1,2,3.0,foo").unwrap_err();
    }
}

#[cfg(test)]
mod four_dp {
    #[test]
    fn ok() {
        let value = super::four_dp("1").unwrap().1;
        assert_eq!(value, 10000);
    }

    #[test]
    fn ok_one_sig_fig() {
        let value = super::four_dp("1.1").unwrap().1;
        assert_eq!(value, 11000);
    }

    #[test]
    fn ok_four_sig_fig() {
        let value = super::four_dp("1.1111").unwrap().1;
        assert_eq!(value, 11111);
    }

    #[test]
    fn err_runthrough() {
        super::four_dp("").unwrap_err();
        super::four_dp("1.").unwrap_err();
    }
}

#[cfg(test)]
mod transaction {

    #[test]
    fn debug() {
        assert_eq!(
            format!("{:?}", super::Transaction::Deposit(1, 1, 2)),
            "Deposit(1, 1, 2)"
        );
        assert_eq!(
            format!("{:?}", super::Transaction::Withdrawal(1, 1, 2)),
            "Withdrawal(1, 1, 2)"
        );
        assert_eq!(
            format!("{:?}", super::Transaction::Dispute(1, 1)),
            "Dispute(1, 1)"
        );
        assert_eq!(
            format!("{:?}", super::Transaction::Resolve(1, 1)),
            "Resolve(1, 1)"
        );
        assert_eq!(
            format!("{:?}", super::Transaction::Chargeback(1, 1)),
            "Chargeback(1, 1)"
        );
    }

    #[test]
    fn partial_eq() {
        assert_eq!(
            super::Transaction::Deposit(1, 1, 20),
            super::Transaction::Deposit(1, 1, 20)
        );
        assert_eq!(
            super::Transaction::Withdrawal(1, 1, 20),
            super::Transaction::Withdrawal(1, 1, 20)
        );
        assert_eq!(
            super::Transaction::Dispute(1, 1),
            super::Transaction::Dispute(1, 1)
        );
        assert_eq!(
            super::Transaction::Resolve(1, 1),
            super::Transaction::Resolve(1, 1)
        );
        assert_eq!(
            super::Transaction::Chargeback(1, 1),
            super::Transaction::Chargeback(1, 1)
        );
    }
}

#[cfg(test)]
mod ws {
    use super::*;
    use nom::bytes::complete::tag;

    #[test]
    fn ok_ws() {
        let (input, tag) = ws(tag::<_, _, ()>("hello"))("  hello  ").unwrap();

        assert_eq!(input, "");
        assert_eq!(tag, "hello");
    }

    #[test]
    fn invalid_inner<'a>() {
        ws(tag("hello"))("").unwrap_err() as nom::Err<(&'a str, nom::error::ErrorKind)>;
    }
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
    fn err_parser_runthrough() {
        parse_header("x").unwrap_err();
        parse_header("type,x").unwrap_err();
        parse_header("type,client,x").unwrap_err();
        parse_header("type,client,tx,x").unwrap_err();
    }

    #[test]
    fn err_extra_value() {
        parse_header("type,client,tx,amount,foo").unwrap_err();
    }
}
