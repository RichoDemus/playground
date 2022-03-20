use std::collections::HashMap;

use bigdecimal::{BigDecimal, Zero};

use crate::transaction::{ClientId, CsvAccount};
use crate::Transaction;

struct Account {
    client_id: ClientId,
    transactions: Vec<Transaction>,
    available: BigDecimal,
    held: BigDecimal,
    locked: bool,
}

impl Account {
    fn new(id: ClientId) -> Self {
        Self {
            client_id: id,
            transactions: vec![],
            available: BigDecimal::zero(),
            held: BigDecimal::zero(),
            locked: false,
        }
    }
    fn process(&mut self, transaction: Transaction) {
        if self.locked {
            // if this was a service, this would be a proper error
            return;
        }
        match transaction {
            Transaction::Deposit { ref amount, .. } => {
                self.available += amount;
            }
            Transaction::Withdrawal { ref amount, .. } => {
                if &self.available >= amount {
                    self.available -= amount;
                } else {
                    // client didn't have enough money for the withdraw
                    // since this is a cli we won't do anything
                    // for a real service this is definitely an
                    // error we want to both log and report to the user
                }
            }
            Transaction::Dispute { tx, .. } => {
                let transactions = self
                    .transactions
                    .iter()
                    .filter(|t| t.tx() == tx)
                    .collect::<Vec<_>>();

                if let [Transaction::Withdrawal { amount, .. }
                | Transaction::Deposit { amount, .. }] = transactions.as_slice()
                {
                    self.available -= amount;
                    self.held += amount;
                }
            }
            Transaction::Resolve { tx, .. } => {
                let transactions = self
                    .transactions
                    .iter()
                    .filter(|t| t.tx() == tx)
                    .collect::<Vec<_>>();

                if let [Transaction::Withdrawal { amount, .. }
                | Transaction::Deposit { amount, .. }, Transaction::Dispute { .. }] =
                    transactions.as_slice()
                {
                    self.available += amount;
                    self.held -= amount;
                }
            }
            Transaction::Chargeback { tx, .. } => {
                let transactions = self
                    .transactions
                    .iter()
                    .filter(|t| t.tx() == tx)
                    .collect::<Vec<_>>();

                if let [Transaction::Withdrawal { amount, .. }
                | Transaction::Deposit { amount, .. }, Transaction::Dispute { .. }, ..] =
                    transactions.as_slice()
                {
                    self.held -= amount;
                    self.locked = true;
                }
            }
        }

        self.transactions.push(transaction);
    }

    fn as_csv_account(&self) -> CsvAccount {
        CsvAccount {
            client: self.client_id,
            available: format!("{:.4}", self.available),
            held: format!("{:.4}", self.held),
            total: format!("{:.4}", (self.available.clone() + self.held.clone())),
            locked: self.locked,
        }
    }
}

pub struct TransactionEngine {
    // I realize this means I'm storing both the client id as the key
    // as well as in the Account struct, I assume that client id can't change
    // but it's still not pretty to store it in two places
    // but I think using a hashmap here is the cleanest
    // and I think  account should store the client id
    accounts: HashMap<ClientId, Account>,
}

impl TransactionEngine {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }

    pub fn process(&mut self, transaction: Transaction) {
        let account = self
            .accounts
            .entry(transaction.client())
            .or_insert_with(|| Account::new(transaction.client()));

        account.process(transaction);
    }

    pub fn accounts(&self) -> Vec<CsvAccount> {
        self.accounts
            .values()
            .map(Account::as_csv_account)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::Transaction::{Chargeback, Deposit, Dispute, Resolve, Withdrawal};

    use super::*;

    #[test]
    fn test_provided_example() {
        let input = vec![
            Deposit {
                client: 1,
                tx: 1,
                amount: BigDecimal::from(1),
            },
            Deposit {
                client: 2,
                tx: 2,
                amount: BigDecimal::from(2),
            },
            Deposit {
                client: 1,
                tx: 3,
                amount: BigDecimal::from(2),
            },
            Withdrawal {
                client: 1,
                tx: 4,
                amount: BigDecimal::from_str("1.5").unwrap(),
            },
            Withdrawal {
                client: 2,
                tx: 5,
                amount: BigDecimal::from(3),
            },
        ];
        let expected = vec![
            CsvAccount {
                client: 1,
                available: "1.5000".to_string(),
                held: "0.0000".to_string(),
                total: "1.5000".to_string(),
                locked: false,
            },
            CsvAccount {
                client: 2,
                available: "2.0000".to_string(),
                held: "0.0000".to_string(),
                total: "2.0000".to_string(),
                locked: false,
            },
        ];
        test(input, expected);
    }

    #[test]
    fn should_deposit_money() {
        test(
            vec![Deposit {
                client: 1,
                tx: 1,
                amount: BigDecimal::from(1),
            }],
            vec![CsvAccount {
                client: 1,
                available: "1.0000".to_string(),
                held: "0.0000".to_string(),
                total: "1.0000".to_string(),
                locked: false,
            }],
        );
    }

    #[test]
    fn should_withdraw_money() {
        test(
            vec![
                Deposit {
                    client: 1,
                    tx: 1,
                    amount: BigDecimal::from(1),
                },
                Withdrawal {
                    client: 1,
                    tx: 2,
                    amount: BigDecimal::from_str("0.5").unwrap(),
                },
            ],
            vec![CsvAccount {
                client: 1,
                available: "0.5000".to_string(),
                held: "0.0000".to_string(),
                total: "0.5000".to_string(),
                locked: false,
            }],
        );
    }

    #[test]
    fn should_not_withdraw_to_below_zero() {
        test(
            vec![
                Deposit {
                    client: 1,
                    tx: 1,
                    amount: BigDecimal::from(1),
                },
                Withdrawal {
                    client: 1,
                    tx: 2,
                    amount: BigDecimal::from(2),
                },
            ],
            vec![CsvAccount {
                client: 1,
                available: "1.0000".to_string(),
                held: "0.0000".to_string(),
                total: "1.0000".to_string(),
                locked: false,
            }],
        );
    }

    #[test]
    fn should_handle_decimals() {
        test(
            vec![
                Deposit {
                    client: 1,
                    tx: 1,
                    amount: BigDecimal::from(1),
                },
                Withdrawal {
                    client: 1,
                    tx: 2,
                    amount: BigDecimal::from_str("0.12345").unwrap(),
                },
                Deposit {
                    client: 1,
                    tx: 1,
                    amount: BigDecimal::from_str("0.12345").unwrap(),
                },
            ],
            vec![CsvAccount {
                client: 1,
                available: "1.0000".to_string(),
                held: "0.0000".to_string(),
                total: "1.0000".to_string(),
                locked: false,
            }],
        );
    }

    #[test]
    fn should_dispute_withdrawal() {
        test(
            vec![
                Deposit {
                    client: 1,
                    tx: 1,
                    amount: BigDecimal::from(1),
                },
                Withdrawal {
                    client: 1,
                    tx: 2,
                    amount: BigDecimal::from_str("0.2").unwrap(),
                },
                Dispute { client: 1, tx: 2 },
            ],
            vec![CsvAccount {
                client: 1,
                available: "0.6000".to_string(),
                held: "0.2000".to_string(),
                total: "0.8000".to_string(),
                locked: false,
            }],
        );
    }

    #[test]
    fn should_ignore_second_dispute() {
        test(
            vec![
                Deposit {
                    client: 1,
                    tx: 1,
                    amount: BigDecimal::from(1),
                },
                Withdrawal {
                    client: 1,
                    tx: 2,
                    amount: BigDecimal::from_str("0.2").unwrap(),
                },
                Dispute { client: 1, tx: 2 },
                Dispute { client: 1, tx: 2 },
            ],
            vec![CsvAccount {
                client: 1,
                available: "0.6000".to_string(),
                held: "0.2000".to_string(),
                total: "0.8000".to_string(),
                locked: false,
            }],
        );
    }

    #[test]
    fn should_resolve_dispute() {
        test(
            vec![
                Deposit {
                    client: 1,
                    tx: 1,
                    amount: BigDecimal::from(1),
                },
                Withdrawal {
                    client: 1,
                    tx: 2,
                    amount: BigDecimal::from_str("0.2").unwrap(),
                },
                Dispute { client: 1, tx: 2 },
                Resolve { client: 1, tx: 2 },
            ],
            vec![CsvAccount {
                client: 1,
                available: "0.8000".to_string(),
                held: "0.0000".to_string(),
                total: "0.8000".to_string(),
                locked: false,
            }],
        )
    }

    #[test]
    fn should_ignore_second_resolve() {
        test(
            vec![
                Deposit {
                    client: 1,
                    tx: 1,
                    amount: BigDecimal::from(1),
                },
                Withdrawal {
                    client: 1,
                    tx: 2,
                    amount: BigDecimal::from_str("0.2").unwrap(),
                },
                Dispute { client: 1, tx: 2 },
                Resolve { client: 1, tx: 2 },
                Resolve { client: 1, tx: 2 },
            ],
            vec![CsvAccount {
                client: 1,
                available: "0.8000".to_string(),
                held: "0.0000".to_string(),
                total: "0.8000".to_string(),
                locked: false,
            }],
        )
    }

    #[test]
    fn should_handle_chargeback() {
        test(
            vec![
                Deposit {
                    client: 1,
                    tx: 1,
                    amount: BigDecimal::from(10),
                },
                Withdrawal {
                    client: 1,
                    tx: 2,
                    amount: BigDecimal::from(2),
                },
                Dispute { client: 1, tx: 2 },
                Chargeback { client: 1, tx: 2 },
            ],
            vec![CsvAccount {
                client: 1,
                available: "6.0000".to_string(),
                held: "0.0000".to_string(),
                total: "6.0000".to_string(),
                locked: true,
            }],
        )
    }

    #[test]
    fn should_ignore_second_chargeback() {
        test(
            vec![
                Deposit {
                    client: 1,
                    tx: 1,
                    amount: BigDecimal::from(10),
                },
                Withdrawal {
                    client: 1,
                    tx: 2,
                    amount: BigDecimal::from(2),
                },
                Dispute { client: 1, tx: 2 },
                Chargeback { client: 1, tx: 2 },
                Chargeback { client: 1, tx: 2 },
            ],
            vec![CsvAccount {
                client: 1,
                available: "6.0000".to_string(),
                held: "0.0000".to_string(),
                total: "6.0000".to_string(),
                locked: true,
            }],
        )
    }

    #[test]
    fn frozen_accounts_should_be_frozen() {
        test(
            vec![
                Deposit {
                    client: 1,
                    tx: 1,
                    amount: BigDecimal::from(10),
                },
                Withdrawal {
                    client: 1,
                    tx: 2,
                    amount: BigDecimal::from(2),
                },
                Dispute { client: 1, tx: 2 },
                Chargeback { client: 1, tx: 2 },
                Deposit {
                    client: 1,
                    tx: 1,
                    amount: BigDecimal::from(10),
                },
            ],
            vec![CsvAccount {
                client: 1,
                available: "6.0000".to_string(),
                held: "0.0000".to_string(),
                total: "6.0000".to_string(),
                locked: true,
            }],
        )
    }

    fn test(transactions: Vec<Transaction>, expected: Vec<CsvAccount>) {
        let mut transation_engine = TransactionEngine::new();
        for transaction in transactions {
            transation_engine.process(transaction);
        }
        let mut result = transation_engine.accounts();
        result.sort_by_key(|a| a.client);
        assert_eq!(result, expected);
    }
}
