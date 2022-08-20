mod parse;

use parse::{parse_header, parse_transactions, Transaction};

fn main() {
    println!("Hello, world!");

    let (_, res) = parse_header("type,client,tx,amount").expect("Error whilst parsing header");
    println!("{:?}", res);

    let (_, res) = parse_transactions("deposit,1,2,3.0").expect("Error whilst parsing header");
    println!("{:?}", res);

    assert_eq!(res, Transaction::Deposit(1, 2, 3.0));
}
