//! # Ledger
//!  This module contains `Ledger`, the state store used for the `csv_ledger` CLI.
//!
//! `Ledger` stores running totals of client accounts, consumes csv files and
//! outputs associated to account statements to string.
//!
//! **Basic example:**
//! ```rust
//! use csv_ledger_lib::ledger::Ledger;
//! use std::{fs::File, io::BufReader};
//! # use std::fs;
//!
//! fn main() {
//!     # fs::write("./foo.csv", "type,client,tx,amount\ndeposit,1,1,1.0").unwrap();
//!     // Read in a new file
//!     let reader = BufReader::new(File::open("./foo.csv").unwrap());
//!     
//!     // Create a new ledger and read in the csv file line by line
//!     let mut ledger = Ledger::default();
//!     ledger.consume_csv(reader);
//!     
//!     // Print out the result
//!     println!("{}", ledger);
//!
//!     # fs::remove_file("./foo.csv").unwrap();
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
    pub clients: HashMap<u16, ClientData>,
    /// The list of transactions. Note: This is a nieve implementation of transaction storage,
    /// requiring all transactions to be stored in memory. Due to there being no maximum limmit to
    /// how old a transaction can be for a `hold` to be applied, all transactions must be addressable.
    pub transactions: BTreeMap<u32, i64>,
}

/// An individual client account.
#[derive(Debug)]
pub struct ClientData {
    held: BTreeMap<u32, i64>,
    available: i64,
    total: i64,
    locked: bool,
}

impl Ledger {
    /// Consume a `BufReader` that contains a csv file of transactions.
    pub fn consume_csv<T>(&mut self, mut reader: BufReader<T>) -> Result<(), LedgerErr>
    where
        T: Read,
    {
        validate_header(&mut reader)?;

        for (index, line) in reader.lines().enumerate() {
            let res = line.map_err(LedgerErr::Reading)?; // map_err is used to provide better debug info
            if !res.trim().is_empty() {
                match parse_transaction(&res)
                    .map_err(|err| LedgerErr::from_parse(err, index + 2))?
                {
                    Transaction::Withdrawal(id, tx, amount) => {
                        self.insert_transaction(id, tx, -amount) // Negative amounts for withdrawals
                    }
                    Transaction::Deposit(id, tx, amount) => self.insert_transaction(id, tx, amount),
                    Transaction::Dispute(id, tx) => self.hold(id, tx),
                    Transaction::Resolve(id, tx) => self.resolve(id, tx),
                    Transaction::Chargeback(id, tx) => self.chageback(id, tx),
                }
            }
        }

        Ok(())
    }

    /// Insert a new transaction
    ///
    /// Example:
    /// ```rust
    /// use csv_ledger_lib::ledger::Ledger;
    ///
    /// // Create a new ledger
    /// let mut ledger = Ledger::default();
    ///
    /// // Deposit
    /// ledger.insert_transaction(1,1,10.0 as i64);
    ///
    /// // Withdrawal
    /// ledger.insert_transaction(1,2,-10.0 as i64);
    /// ```
    pub fn insert_transaction(&mut self, client_id: u16, transaction_id: u32, amount: i64) {
        if let Some(client) = self.clients.get_mut(&client_id) {
            if !client.locked {
                client.total += amount;
                client.available += amount;
                self.transactions.insert(transaction_id, amount);
            }
        } else {
            self.clients.insert(client_id, ClientData::new(amount));
            self.transactions.insert(transaction_id, amount);
        }
    }

    /// Opens a dispute on a transaction.
    pub fn hold(&mut self, client_id: u16, transaction_id: u32) {
        // Discard any incorrect inputs
        if let Some(client) = self.clients.get_mut(&client_id) {
            if let Some(amount) = self.transactions.remove(&transaction_id) {
                {
                    client.available -= amount;
                    client.held.insert(transaction_id, amount);
                }
            }
        }
    }

    /// Resolves a disputed transaction - adds disputed transaction's value back to the available funds.
    pub fn resolve(&mut self, client_id: u16, transaction_id: u32) {
        // Discard any incorrect inputs
        if let Some(client) = self.clients.get_mut(&client_id) {
            if let Some(amount) = client.held.remove(&transaction_id) {
                client.available += amount;
            }
        }
    }

    /// Peform a chargeback on a disputed transaction -
    pub fn chageback(&mut self, client_id: u16, transaction_id: u32) {
        // Discard any incorrect inputs
        if let Some(client) = self.clients.get_mut(&client_id) {
            if let Some(amount) = client.held.remove(&transaction_id) {
                client.total -= amount;
                client.locked = true;
            }
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

/// Validate the header of the csv file.
fn validate_header<T>(reader: &mut BufReader<T>) -> Result<(), LedgerErr>
where
    T: Read,
{
    let mut buf = String::new();
    reader.read_line(&mut buf).map_err(LedgerErr::Reading)?; // map_err is used to provide better debug info
    parse_header(&buf).map_err(|err| LedgerErr::Parse(err.to_string(), 1))?;
    Ok(())
}

impl ClientData {
    fn new(amount: i64) -> Self {
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
            dp_string(self.available),
            dp_string(self.held.values().sum()),
            dp_string(self.total),
            self.locked
        )
    }
}

/// Convert a i64 to a string with four decimal places (eg val / 100)
fn dp_string(amount: i64) -> String {
    format!("{}.{:04}", amount / 10000, amount % 10000)
}

#[cfg(test)]
mod dp_string {
    use super::dp_string;
    #[test]
    fn test_dp_string() {
        assert_eq!(dp_string(0), "0.0000");
        assert_eq!(dp_string(1), "0.0001");
        assert_eq!(dp_string(10), "0.0010");
        assert_eq!(dp_string(100), "0.0100");
        assert_eq!(dp_string(1000), "0.1000");
        assert_eq!(dp_string(10000), "1.0000");
    }
}

#[cfg(test)]
mod validate_header {
    use super::validate_header;
    use std::io::{BufReader, Cursor, Error, ErrorKind, Read};

    struct TestReader {}

    impl Read for TestReader {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Err(Error::new(ErrorKind::InvalidData, "Something went wrong."))
        }
    }

    #[test]
    fn ok() {
        validate_header(&mut BufReader::new(Cursor::new("type, client, tx, amount"))).unwrap();
    }

    #[test]
    fn err_runthrough() {
        validate_header(&mut BufReader::new(TestReader {})).unwrap_err();
        validate_header(&mut BufReader::new(Cursor::new(""))).unwrap_err();
        validate_header(&mut BufReader::new(Cursor::new("\n"))).unwrap_err();
        validate_header(&mut BufReader::new(Cursor::new("type,"))).unwrap_err();
    }
}

#[cfg(test)]
mod client_data {
    use super::ClientData;

    #[test]
    fn debug() {
        let data = ClientData::new(10);

        assert_eq!(
            format!("{:?}", data),
            "ClientData { held: {}, available: 10, total: 10, locked: false }"
        );
    }
}

#[cfg(test)]
mod ledger {
    use super::{ClientData, Ledger};
    use std::collections::BTreeMap;
    use std::io::{BufReader, Cursor, Error, ErrorKind, Read};

    struct TestReader {}

    impl Read for TestReader {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Err(Error::new(ErrorKind::InvalidData, "Something went wrong."))
        }
    }

    struct TestReaderTwo<'a> {
        inner: Cursor<&'a str>,
        state: bool,
    }

    // Fail after second read
    impl Read for TestReaderTwo<'_> {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if self.state {
                Err(Error::new(ErrorKind::InvalidData, "Something went wrong."))
            } else {
                self.state = true;
                Ok(self.inner.read(buf).unwrap())
            }
        }
    }

    #[test]
    fn ok_consume() {
        let mut ledger = Ledger::default();

        ledger
            .consume_csv(BufReader::new(Cursor::new(
                "type, client, tx, amount
                
                deposit, 1, 1, 20.0
                withdrawal,1,2,10.0
                dispute,1,2,
                resolve,1,2,
            
                deposit,2,3,113.1112
                dispute,2,3,
                chargeback,2,3,
                
                ",
            )))
            .unwrap();

        let result = ledger.to_string();
        let mut lines = result.lines();

        assert_eq!(
            lines.next().unwrap(),
            "client, available, held, total, locked"
        );

        let accounts = vec![
            "1, 10.0000, 0.0000, 10.0000, false",
            "2, 0.0000, 0.0000, 0.0000, true",
        ];

        assert!(accounts.contains(&lines.next().unwrap()));
        assert!(accounts.contains(&lines.next().unwrap()));
        assert!(lines.next().is_none())
    }

    #[test]
    fn err_consume_runthrough() {
        let mut ledger = Ledger::default();

        ledger
            .consume_csv(BufReader::new(Cursor::new("")))
            .unwrap_err();

        ledger
            .consume_csv(BufReader::new(Cursor::new(&[0x0])))
            .unwrap_err();

        ledger
            .consume_csv(BufReader::new(TestReader {}))
            .unwrap_err();

        ledger
            .consume_csv(BufReader::new(TestReaderTwo {
                inner: Cursor::new("type, client, tx, amount\n"),
                state: false,
            }))
            .unwrap_err();

        ledger
            .consume_csv(BufReader::new(Cursor::new("type, client, tx, amount\n123")))
            .unwrap_err();
    }

    #[test]
    fn insert_transaction() {
        let mut client_2 = ClientData::new(0);
        client_2.locked = true;

        let mut ledger = Ledger {
            clients: [(2_u16, client_2)].into_iter().collect(),
            transactions: BTreeMap::new(),
        };

        ledger.insert_transaction(1, 1, 1);
        ledger.insert_transaction(1, 2, 1);

        // Locked
        ledger.insert_transaction(2, 3, 1);

        let client_1 = ledger.clients.get(&1).unwrap();
        let client_2 = ledger.clients.get(&2).unwrap();
        assert_eq!(client_1.available, 2);
        assert_eq!(client_2.available, 0);
        assert_eq!(client_1.total, 2);
        assert_eq!(client_2.total, 0);
    }

    #[test]
    fn dispute() {
        let mut ledger = Ledger::default();

        ledger.insert_transaction(1, 1, 1);
        ledger.hold(1, 1);
        ledger.hold(2, 1);
        ledger.hold(1, 2);

        let c = ledger.clients.get(&1).unwrap();

        assert_eq!(ledger.clients.len(), 1);
        assert_eq!(c.held.get(&1).unwrap(), &1_i64);
        assert_eq!(c.available, 0_i64);
    }

    #[test]
    fn resolve() {
        let mut ledger = Ledger::default();

        ledger.insert_transaction(1, 1, 1);
        ledger.hold(1, 1);
        ledger.resolve(1, 1);
        ledger.resolve(2, 1);
        ledger.resolve(1, 2);

        let c = ledger.clients.get(&1).unwrap();
        assert_eq!(c.held.len(), 0);
        assert_eq!(c.available, 1_i64);
    }

    #[test]
    fn chargeback() {
        let mut ledger = Ledger::default();

        ledger.insert_transaction(1, 1, 1);
        ledger.hold(1, 1);
        ledger.chageback(1, 1);
        ledger.chageback(2, 1);
        ledger.chageback(1, 2);

        let c = ledger.clients.get(&1).unwrap();
        assert_eq!(c.held.len(), 0);
        assert_eq!(c.total, 0_i64);
        assert_eq!(c.locked, true);
    }

    #[test]
    fn debug() {
        assert_eq!(
            format!("{:?}", Ledger::default()),
            "Ledger { clients: {}, transactions: {} }"
        )
    }

    #[test]
    fn display() {
        let mut ledger = Ledger::default();
        ledger.insert_transaction(1, 1, 1);
        assert_eq!(
            format!("{}", ledger),
            "client, available, held, total, locked\n1, 0.0001, 0.0000, 0.0001, false"
        );
    }
}
