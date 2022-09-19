use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::{Input, TxType, DECIMALS};
use log::warn;

#[derive(Debug)]
pub struct Account {
    pub locked: bool,
    pub available: Decimal,
    pub held: Decimal,
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
    pub fn total(&self) -> Decimal {
        self.available + self.held
    }
}

#[derive(Debug)]
pub struct Transaction {
    pub client: u16,
    pub tx_type: TxType,
    pub amount: Decimal,
    pub under_dispute: bool,
}

/// Process records and update client accounts and transactions accordingly
pub fn process_record(
    client_accounts: &mut HashMap<u16, Account>,
    transactions: &mut HashMap<u32, Transaction>,
    record: Input,
) {
    // create entry if not found
    let client_account = client_accounts
        .entry(record.client)
        .or_insert_with(Account::default);

    if client_account.locked {
        warn!(
            "Client account {} is locked. Cannot process transaction.",
            record.client
        );
        return;
    }

    if let Some(amount) = record.amount {
        // Save a copy of the transaction if it contains an amount (deposit or withdrawal) for disputes
        // Note, we assume 4 decimals here, so we don't raise nor rescale.
        // but it should be done if we don't fully trust the input
        // record.amount.rescale(4)
        let new_tx = Transaction {
            client: record.client,
            tx_type: record.r#type.clone(),
            amount,
            under_dispute: false,
        };
        transactions.insert(record.tx, new_tx);
    };

    match record.r#type {
        TxType::Deposit => {
            // Note unwrap is safe here because of the type matching
            // however to be more thorough, a check on the option should be done (same for withdrawal)
            client_account.available += record.amount.unwrap();
        }
        TxType::Withdrawal => {
            // Only withdraw if amount in account
            if client_account.available >= record.amount.unwrap() {
                client_account.available -= record.amount.unwrap();
            } else {
                // log invalid withdrawal
                warn!(
                    "Unable to withdraw amount: {} from account {}",
                    record.amount.unwrap(),
                    record.client
                );
            }
        }
        TxType::Dispute => {
            if let Some(tx) = transactions.get_mut(&record.tx) {
                // Note make sure that there are enough funds to dispute a transaction was a deposit,
                // and make sure that the transaction was a deposit otherwise it would allow system abuse
                if tx.tx_type == TxType::Deposit && client_account.available >= tx.amount {
                    client_account.available -= tx.amount;
                    client_account.held += tx.amount;
                    tx.under_dispute = true;
                } else {
                    warn!("Not enough found or previous transaction was not a deposit for transaction {}", record.tx);
                }
            } else {
                // Note it's possible that the tx doesn't exist, in that case ignore and log
                warn!("Unable to find transaction for dispute {}", record.tx);
            }
        }
        TxType::Resolve => {
            if let Some(tx) = transactions.get_mut(&record.tx) {
                // Note: only resolve transactions that were under dispute
                if tx.under_dispute {
                    client_account.available += tx.amount;
                    client_account.held -= tx.amount;
                    tx.under_dispute = false;
                } else {
                    warn!(
                        "Transaction {} was not under dispute, cannot resolve.",
                        record.tx
                    );
                }
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
                        client_account.locked = true
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

#[cfg(test)]
mod tests {
    use crate::processor::process_record;
    use crate::{Account, Input, Transaction, TxType};
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    #[test]
    /// Add funds to a client account
    fn add_funds() {
        let mut client_accounts: HashMap<u16, Account> = HashMap::new();
        let account = Account {
            locked: false,
            available: dec!(1),
            held: dec!(0),
        };
        client_accounts.insert(1, account);
        let mut transactions: HashMap<u32, Transaction> = HashMap::new();
        //let tx = Transaction { amount: , under_dispute: false };
        let input = Input {
            r#type: TxType::Deposit,
            client: 1,
            tx: 10,
            amount: Some(dec!(10.0004)),
        };
        process_record(&mut client_accounts, &mut transactions, input);
        // Check that the client account is correct
        let updated_account = client_accounts.get(&1).unwrap();
        assert_eq!(updated_account.available, dec!(11.0004));
    }

    #[test]
    /// Add funds to a locked account - nothing happens
    fn add_funds_locked() {
        let mut client_accounts: HashMap<u16, Account> = HashMap::new();
        let initial_amount = dec!(1);
        let account = Account {
            locked: true,
            available: initial_amount,
            held: dec!(0),
        };
        client_accounts.insert(1, account);
        let mut transactions: HashMap<u32, Transaction> = HashMap::new();
        //let tx = Transaction { amount: , under_dispute: false };
        let input = Input {
            r#type: TxType::Deposit,
            client: 1,
            tx: 10,
            amount: Some(dec!(10.0004)),
        };
        process_record(&mut client_accounts, &mut transactions, input);
        // Check that the available funds did not change
        let updated_account = client_accounts.get(&1).unwrap();
        assert_eq!(updated_account.available, initial_amount);
    }

    #[test]
    /// Withdraw funds from a client account
    fn withdraw_funds() {
        let mut client_accounts: HashMap<u16, Account> = HashMap::new();
        let account = Account {
            locked: false,
            available: dec!(10.2),
            held: dec!(0),
        };
        client_accounts.insert(1, account);
        let mut transactions: HashMap<u32, Transaction> = HashMap::new();
        //let tx = Transaction { amount: , under_dispute: false };
        let input = Input {
            r#type: TxType::Withdrawal,
            client: 1,
            tx: 10,
            amount: Some(dec!(10.0004)),
        };
        process_record(&mut client_accounts, &mut transactions, input);
        // Check that the client account is correct
        let updated_account = client_accounts.get(&1).unwrap();
        assert_eq!(updated_account.available, dec!(0.1996));
    }

    #[test]
    /// Dispute funds from a client account
    fn chargeback_funds() {
        let mut client_accounts: HashMap<u16, Account> = HashMap::new();
        let account = Account {
            locked: false,
            available: dec!(0),
            held: dec!(0),
        };
        client_accounts.insert(1, account);
        let mut transactions: HashMap<u32, Transaction> = HashMap::new();

        // Add a deposit
        let deposit = Input {
            r#type: TxType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(dec!(10)),
        };
        process_record(&mut client_accounts, &mut transactions, deposit);

        // Add a dispute
        let dispute = Input {
            r#type: TxType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        };
        process_record(&mut client_accounts, &mut transactions, dispute);

        // Add a chargeback
        let chargeback = Input {
            r#type: TxType::Chargeback,
            client: 1,
            tx: 1,
            amount: None,
        };
        process_record(&mut client_accounts, &mut transactions, chargeback);

        // Check that the client account is now locked
        let updated_account = client_accounts.get(&1).unwrap();
        assert_eq!(updated_account.locked, true);
    }
}
