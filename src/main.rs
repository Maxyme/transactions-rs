/// Transaction-rs.
use anyhow::Result;
use csv::{ReaderBuilder, Trim, Writer};
use std::collections::HashMap;
use std::{env, io};

use log::warn;
use rust_decimal::prelude::*;
use serde::Serialize;
use std::path::PathBuf;

const DECIMALS: u32 = 4;

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

#[derive(Serialize, Debug)]
struct Output {
    client: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
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
            available: Decimal::new(0, DECIMALS),
            held: Decimal::new(0, DECIMALS),
        }
    }
}

impl Account {
    fn total(&self) -> Decimal {
        self.available + self.held
    }
}

#[derive(Debug)]
struct Transaction {
    amount: Decimal,
    under_dispute: bool,
}

fn main() -> Result<()> {
    // Parse input args. Note this can be more robust with a package like `clap`
    let args: Vec<String> = env::args().collect();
    let path = PathBuf::from(&args[1]);

    // Create a reader that trims whitespaces
    let mut reader = ReaderBuilder::new().trim(Trim::All).from_path(path)?;

    // Store client accounts and transactions in separate hashmaps
    // Note: these could be stored in a database on disk if we need more memory that what is available
    let mut client_accounts: HashMap<u16, Account> = HashMap::new();
    let mut transactions: HashMap<u32, Transaction> = HashMap::new();

    for record in reader.deserialize() {
        process_record(&mut client_accounts, &mut transactions, record?)
    }

    // Output the client account balances to stdout
    let mut wtr = Writer::from_writer(io::stdout());
    output_accounts(&client_accounts, &mut wtr)?;

    Ok(())
}

fn output_accounts<W>(client_accounts: &HashMap<u16, Account>, writer: &mut Writer<W>) -> Result<()>
where
    W: io::Write,
{
    /// Output account with desired writer (note, this could be to a csv file or other output)
    for (client_id, account) in client_accounts {
        // Output values with four places past the decimal
        writer.serialize(Output {
            client: *client_id,
            available: account.available,
            held: account.held,
            total: account.total(),
            locked: account.locked,
        })?;
    }
    writer.flush()?;
    Ok(())
}

fn process_record(
    client_accounts: &mut HashMap<u16, Account>,
    transactions: &mut HashMap<u32, Transaction>,
    record: Input,
) {
    /// Process records and update client accounts and transactions accordingly
    // Save a copy of the transaction for disputes
    // Note, we assume 4 decimals here, so we don't raise nor rescale.
    // but it should be done if we don't fully trust the input
    // record.amount.rescale(4)
    
    let new_tx = Transaction {
        amount: record.amount,
        under_dispute: false,
    };
    transactions.insert(record.tx, new_tx);

    // create entry if not found
    let client_account = client_accounts
        .entry(record.client)
        .or_insert_with(Account::default);
    match record.r#type {
        TxType::Deposit => {
            client_account.available += record.amount;
        }
        TxType::Withdrawal => {
            // Only withdraw if amount in account
            if client_account.available >= record.amount {
                client_account.available -= record.amount;
            } else {
                // log invalid withdrawal
                warn!(
                    "Unable to withdraw amount: {} from account {}",
                    record.amount, record.client
                );
            }
        }
        TxType::Dispute => {
            if let Some(tx) = transactions.get_mut(&record.tx) {
                // todo: should we make sure that the transaction was a deposit, otherwise it would allow system abuse
                client_account.available -= tx.amount;
                client_account.held += tx.amount;
                tx.under_dispute = true;
            } else {
                // Note it's possible that the tx doesn't exist, in that case ignore and log
                warn!("Unable to find transaction for dispute {}", record.tx);
            }
        }
        TxType::Resolve => {
            if let Some(tx) = transactions.get_mut(&record.tx) {
                // todo: should we make sure that the transaction was a deposit, otherwise it would allow system abuse
                client_account.available += tx.amount;
                client_account.held -= tx.amount;
                tx.under_dispute = false;
            } else {
                // Note it's possible that the tx doesn't exist, in that case ignore and log
                warn!("Unable to find transaction to resolve {}", record.tx);
            }
        }
        TxType::Chargeback => {
            if let Some(tx) = transactions.get(&record.tx) {
                if tx.under_dispute {
                    if client_account.held >= tx.amount {
                        client_account.held -= tx.amount;
                    } else {
                        // log invalid chargeback
                        warn!(
                            "Invalid chargeback amount {} for transaction {}",
                            tx.amount, record.tx
                        )
                    }
                } else {
                    // log invalid resolve status
                    warn!(
                        "Cannot chargeback transaction {} as it's not under dispute",
                        record.tx
                    );
                }
            }
        }
    }
}
