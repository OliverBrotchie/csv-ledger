use crate::{
    parse::{parse_header, parse_transaction, Transaction},
    NomErr,
};
use std::{
    collections::{BTreeMap, HashMap},
    fmt::{self, Display},
    fs::File,
    io::{self, BufRead, BufReader},
};

#[derive(Debug)]
pub enum BankErr {
    Io(String),
    Parse(String),
}

impl From<io::Error> for BankErr {
    fn from(err: io::Error) -> Self {
        BankErr::Io(err.to_string())
    }
}

impl<E> From<NomErr<E>> for BankErr {
    fn from(err: NomErr<E>) -> Self {
        BankErr::Parse(
            match err {
                NomErr::Incomplete(_) => "Input was incomplete.",
                NomErr::Error(_) => "Input was in the wrong format.",
                NomErr::Failure(_) => "Faliure whilst parsing input.",
            }
            .to_string(),
        )
    }
}

#[derive(Default, Debug)]
pub struct Bank {
    clients: HashMap<u16, ClientData>,
    transactions: HashMap<u32, f64>,
}

#[derive(Debug)]
pub struct ClientData {
    held: BTreeMap<u32, f64>,
    available: f64,
    total: f64,
    locked: bool,
}

impl Bank {
    pub fn consume_csv(&mut self, mut reader: BufReader<File>) -> Result<(), BankErr> {
        validate_header(&mut reader)?;

        for line in reader.lines() {
            match parse_transaction(&line?)?.1 {
                Transaction::Withdrawal(id, tx, amount) => self.add_transaction(id, tx, -amount),
                Transaction::Deposit(id, tx, amount) => self.add_transaction(id, tx, amount),
                Transaction::Dispute(id, tx) => self.hold(id, tx),
                Transaction::Resolve(id, tx) => self.resolve(id, tx),
                Transaction::Chargeback(id, tx) => self.chageback(id, tx),
            }
        }

        Ok(())
    }

    fn add_transaction(&mut self, client_id: u16, transaction_id: u32, amount: f64) {
        if let Some(client) = self.clients.get_mut(&client_id) && !client.locked {
            client.available += amount;
            client.total += amount;
            self.transactions.insert(transaction_id, amount);
        } else {
            self.clients.insert(client_id, ClientData::new(amount));
            self.transactions.insert(transaction_id, amount);
        }
    }

    fn hold(&mut self, client_id: u16, transaction_id: u32) {
        if let (Some(amount), Some(client)) = (
            self.transactions.get(&transaction_id),
            self.clients.get_mut(&client_id),
        ) {
            client.available -= amount;
            client.held.insert(transaction_id, *amount);
        }
    }

    fn resolve(&mut self, client_id: u16, transaction_id: u32) {
        if let Some(client) = self.clients.get_mut(&client_id) {
            if let Some(amount) = client.held.remove(&transaction_id) {
                client.available += amount;
            }
        }
    }

    fn chageback(&mut self, client_id: u16, transaction_id: u32) {
        if let Some(client) = self.clients.get_mut(&client_id) {
            if let Some(amount) = client.held.remove(&transaction_id) {
                client.total -= amount;
                client.locked = true;
            }
        }
    }
}

fn validate_header(reader: &mut BufReader<File>) -> Result<(), BankErr> {
    let mut buf = String::new();
    reader.read_line(&mut buf)?;
    parse_header(&buf)?;
    Ok(())
}

impl Display for Bank {
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
