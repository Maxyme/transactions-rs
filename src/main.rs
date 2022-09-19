/// Transaction-rs.
use anyhow::Result;
use csv::{ReaderBuilder, Trim, Writer};
use std::collections::HashMap;
use std::{env, io};

use rust_decimal::prelude::*;

use std::path::PathBuf;

use serde::Serialize;

use processor::{Account, Transaction};
use serde::Deserialize;

mod processor;

const DECIMALS: u32 = 4;

#[derive(Deserialize, Debug)]
pub enum TxType {
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
pub struct Output {
    client: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

#[derive(Deserialize, Debug)]
pub struct Input {
    r#type: TxType,
    client: u16,
    tx: u32,
    amount: Option<Decimal>,
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
        let input = record?;
        processor::process_record(&mut client_accounts, &mut transactions, input)
    }

    // Output the client account balances to stdout
    let mut wtr = Writer::from_writer(io::stdout());
    output_accounts(&client_accounts, &mut wtr)?;

    Ok(())
}

/// Output account with desired writer (note, this could be to a csv file or other output)
fn output_accounts<W>(client_accounts: &HashMap<u16, Account>, writer: &mut Writer<W>) -> Result<()>
where
    W: io::Write,
{
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
