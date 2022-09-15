use std::collections::HashMap;
/// Transaction-rs.
use std::path::PathBuf;
use csv::{ReaderBuilder, Trim};
use anyhow::Result;
// Using the prelude can help importing trait based functions (e.g. core::str::FromStr).
use rust_decimal::prelude::*;

#[derive(Deserialize, Debug)]
enum TxType {
    #[serde(rename = "deposit")]
    Deposit,
    #[serde(rename = "withdrawal")]
    Withdrawal,
    #[serde(rename = "dispute")]
    Dispute,
    #[serde(rename = "resolve")]
    Resolve,
    #[serde(rename = "chargeback")]
    Chargeback,
}

use serde::Serialize;
#[derive(Deserialize, Debug)]
struct Output {
    client: u16,
    available: Decimal,
    held: Decimal,
    // todo: output to 4 decimals no matter what
    total: Decimal,
    locked: bool
}

use serde::Deserialize;
#[derive(Deserialize, Debug)]
struct Input {
    r#type: TxType,
    client: u16,
    tx: u32,
    // Todo - check for up to 4 decimals
    amount: Decimal,
}
#[derive(Debug)]
struct Account {
    locked: bool,
    available: Decimal,
    held: Decimal,
}

impl Default for Account {
    fn default() -> Account {
        Account {
            locked: false,
            available: Decimal::from(0),
            held: Decimal::from(0)
            //..Default::default()
        }
    }
}

impl Account {
    fn total(&self) -> Decimal {
        self.available + self.held
    }
}

struct Transaction {
    amount: Decimal,
    under_dispute: bool
}

// Todo add new instead with amount specified
impl Default for Transaction {
    fn default() -> Transaction {
        Transaction {
            under_dispute: false,
            amount: Decimal::from(0),
        }
    }
}


fn main() -> Result<()> {
    //println!("Hello, world!");
    // todo pass path as arg
    let path = PathBuf::from("example.csv");

    // Create a reader that will trim whitespaces
    let mut reader = ReaderBuilder::new().trim(Trim::All).from_path(path)?;

    let mut client_accounts: HashMap<u16, Account> = HashMap::new();
    let mut transactions: HashMap<u32, Transaction> = HashMap::new();
    // todo: fill hashmap
    // todo: confirm stream read
    for record in reader.deserialize() {
        let record: Input = record?;
        println!("{:?}", record);
        // Save a copy of the transaction
        let mut new_tx = Transaction::default();
        new_tx.
        transactions.insert(record.tx, new_tx);

        // create entry if not found
        let client_account = client_accounts.entry(record.client).or_insert_with(|| Account::default());
        match record.r#type {
            TxType::Deposit => {
                client_account.available += record.amount;
            }
            TxType::Withdrawal => {
                // Only withdraw if amount in account
                if client_account.available >= record.amount {
                    client_account.available -= record.amount;
                }
            }
            TxType::Dispute => {
                // Note it's possible that the tx doesn't exist, in that case ignore
                match transactions.get(&record.tx) {
                    Some(amount) => {
                        // todo: make sure that the transaction was a deposit, otherwise it would allow system abuse

                        client_account.available -= amount;
                        client_account.held += amount;
                        // todo: Set transaction in dispute to true
                    },
                    None => ()
                }


            }
            TxType::Resolve => {
                // todo: check that the transaction was actually under dispute
                match transactions.get(&record.tx) {
                    Some(tx) => {
                        client_account.available += amount;
                        client_account.held -= amount;
                    },
                    None => ()
                }
            }
            TxType::Chargeback => {}
        }
    }
    println!("{:?}", transactions);
    println!("{:?}", client_accounts);
    // Output the client account balances
    Ok(())
}
