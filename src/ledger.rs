//! # Ledger
//!  This module contains `Ledger`, the state store used for this CLI.
//!
//! `Ledger` stores running totals of client accounts, consumes csv files and
//! outputs associated to account statements to string.
//!
//! **Basic example:**
//! ```rust
//! use crate::ledger::ledger
//!
//! fn main() {
//!     // Read in a new file
//!     let reader = BufReader::new(File::open("./foo.csv").unwrap());
//!     
//!     // Create a new ledger and read in the csv file line by line
//!     let ledger = Ledger::default();
//!     ledger.consume_csv(reader);
//!     
//!     // Print out the result
//!     println!("{}", ledger);
//! }
//! ```

use crate::{
    parse::{parse_header, parse_transaction, Transaction},
    LedgerErr,
};
use std::{
    collections::{BTreeMap, HashMap},
    fmt::{self, Display},
    io::{BufRead, BufReader, Read},
};

#[derive(Default, Debug)]
pub struct Ledger {
    /// The list of client accounts.
    clients: HashMap<u16, ClientData>,
    /// The list of transactions. Note: This is a nieve implementation of transaction storage,
    /// requiring all transactions to be stored in memory. Due to there being no maximum limmit to
    /// how old a transaction can be for a `hold` to be applied, all transactions must be addressable.
    transactions: HashMap<u32, f64>,
}

/// An individual client account.
#[derive(Debug)]
pub struct ClientData {
    held: BTreeMap<u32, f64>,
    available: f64,
    total: f64,
    locked: bool,
}

impl Ledger {
    /// Consume a `BufReader` that contains a csv file of transactions.
    pub fn consume_csv<T>(&mut self, mut reader: BufReader<T>) -> Result<(), LedgerErr>
    where
        T: Read,
    {
        validate_header(&mut reader)?;

        for line in reader.lines() {
            match parse_transaction(&line.map_err(LedgerErr::Reading)?)?.1 {
                Transaction::Withdrawal(id, tx, amount) => self.insert_transaction(id, tx, -amount),
                Transaction::Deposit(id, tx, amount) => self.insert_transaction(id, tx, amount),
                Transaction::Dispute(id, tx) => self.hold(id, tx),
                Transaction::Resolve(id, tx) => self.resolve(id, tx),
                Transaction::Chargeback(id, tx) => self.chageback(id, tx),
            }
        }

        Ok(())
    }

    /// Insert a new transaction
    ///
    /// Example:
    /// ```rust
    /// const ledger = Ledger::default();
    ///
    /// // Deposit
    /// ledger.insert_transaction(1,1,10);
    ///
    /// // Withdrawal
    /// ledger.insert_transaction(1,2,-10.0);
    /// ```
    fn insert_transaction(&mut self, client_id: u16, transaction_id: u32, amount: f64) {
        if let Some(client) = self.clients.get_mut(&client_id) && !client.locked {
            client.available += amount;
            client.total += amount;
            self.transactions.insert(transaction_id, amount);
        } else {
            self.clients.insert(client_id, ClientData::new(amount));
            self.transactions.insert(transaction_id, amount);
        }
    }

    /// Opens a dispute on a transaction.
    fn hold(&mut self, client_id: u16, transaction_id: u32) {
        // Discard any incorrect inputs
        if let (Some(amount), Some(client)) = (
            self.transactions.get(&transaction_id),
            self.clients.get_mut(&client_id),
        ) {
            client.available -= amount;
            client.held.insert(transaction_id, *amount);
        }
    }

    /// Resolves a disputed transaction - adds disputed transaction's value back to the available funds.
    fn resolve(&mut self, client_id: u16, transaction_id: u32) {
        // Discard any incorrect inputs
        if let Some(client) = self.clients.get_mut(&client_id) &&
            let Some(amount) = client.held.remove(&transaction_id)
        {
            client.available += amount;
        }
    }

    /// Peform a chargeback on a disputed transaction -
    fn chageback(&mut self, client_id: u16, transaction_id: u32) {
        // Discard any incorrect inputs
        if let Some(client) = self.clients.get_mut(&client_id) &&
            let Some(amount) = client.held.remove(&transaction_id)
        {
            client.total -= amount;
            client.locked = true;
        }
    }
}

impl Display for Ledger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "client, available, held, total, locked{}",
            self.clients
                .iter()
                .fold(String::new(), |acc, (key, value)| format!(
                    "{acc}\n{key}, {value}"
                ))
        )
    }
}

fn validate_header<T>(reader: &mut BufReader<T>) -> Result<(), LedgerErr>
where
    T: Read,
{
    let mut buf = String::new();
    reader.read_line(&mut buf).map_err(LedgerErr::Reading)?;
    parse_header(&buf)?;
    Ok(())
}

impl ClientData {
    fn new(amount: f64) -> Self {
        ClientData {
            held: BTreeMap::new(),
            available: amount,
            total: amount,
            locked: false,
        }
    }
}

impl Display for ClientData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}",
            self.available,
            self.held.values().sum::<f64>(),
            self.total,
            self.locked
        )
    }
}

#[cfg(test)]
mod validate_header {}
