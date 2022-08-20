extern crate nom;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{multispace0, u16, u32},
    combinator::opt,
    error::ParseError,
    number::complete::double,
    sequence::{delimited, terminated},
    IResult,
};

#[derive(Debug, PartialEq)]
pub enum Transaction {
    Deposit(u16, u32, f64),
    Withdrawal(u16, u32, f64),
    Dispute(u16, u32),
    Resolve(u16, u32),
    Chargeback(u16, u32),
}

fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

pub fn parse_header(input: &str) -> IResult<&str, ()> {
    let (input, _) = terminated(ws(tag("type")), tag(","))(input)?;
    let (input, _) = terminated(ws(tag("client")), tag(","))(input)?;
    let (input, _) = terminated(ws(tag("tx")), tag(","))(input)?;
    Ok((ws(tag("amount"))(input)?.0, ()))
}

pub fn parse_transactions(input: &str) -> IResult<&str, Transaction> {
    // Parse the type of transaction
    let (input, key) = terminated(
        ws(alt((
            tag("deposit"),
            tag("withdrawal"),
            tag("dispute"),
            tag("resolve"),
            tag("chageback"),
        ))),
        tag(","),
    )(input)?;

    // Parse the account and transaction ID
    let (input, client) = terminated(ws(u16), tag(","))(input)?;
    let (input, tx) = terminated(ws(u32), tag(","))(input)?;

    // Parse the transaction amount
    let (input, amount) = opt(ws(double))(input)?;

    Ok((
        input,
        match (key, amount) {
            ("deposit", Some(value)) => Transaction::Deposit(client, tx, value),
            ("withdrawal", Some(value)) => Transaction::Withdrawal(client, tx, value),
            ("dispute", None) => Transaction::Dispute(client, tx),
            ("resolve", None) => Transaction::Resolve(client, tx),
            ("chargeback", None) => Transaction::Chargeback(client, tx),
            _ => unimplemented!(),
        },
    ))
}

#[cfg(test)]

mod parse_header {
    use crate::parse::parse_header;

    #[test]
    fn ok_no_white_space() {
        let (rem, _) = parse_header("type,client,tx,amount").expect("Error whilst parsing header.");
        assert_eq!(rem, "");
    }

    #[test]
    fn ok_with_white_space() {
        let (rem, _) = parse_header("   type    ,  client,   tx  ,    amount    ")
            .expect("Error whilst parsing header.");

        assert_eq!(rem, "");
    }

    #[test]
    fn err_invalid_input() {
        parse_header("client,type,ammount,tx").unwrap_err();
    }

    #[test]
    fn err_missing_value() {
        parse_header("type,client,tx,").unwrap_err();
    }
}

#[cfg(test)]
mod parse_transactions {
    use crate::parse::{parse_transactions, Transaction};

    #[test]
    fn ok_no_white_space() {
        let (rem, res) =
            parse_transactions("deposit,1,2,3.0").expect("Error whilst parsing header.");

        assert_eq!(res, Transaction::Deposit(1, 2, 3.0));
        assert_eq!(rem, "");
    }

    #[test]
    fn ok_with_white_space() {
        let (rem, res) = parse_transactions("   deposit   ,1  ,   2,  3.0  ")
            .expect("Error whilst parsing transaction.");
        assert_eq!(res, Transaction::Deposit(1, 2, 3.0));
        assert_eq!(rem, "");
    }

    #[test]
    fn ok_no_amount() {
        let (rem, res) =
            parse_transactions("dispute,1,2,").expect("Error whilst parsing transaction.");
        assert_eq!(res, Transaction::Dispute(1, 2));
        assert_eq!(rem, "");
    }

    #[test]
    fn err_invalid_u16() {
        parse_transactions("deposit,65536,2,3.0").unwrap_err();
    }

    #[test]
    fn err_invalid_deposit() {
        parse_transactions("deposit,1,2,").unwrap_err();
    }

    #[test]
    fn err_invalid_dispute() {
        parse_transactions("dispute,1,").unwrap_err();
    }
}