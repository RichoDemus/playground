use bigdecimal::BigDecimal;
use serde::Deserialize;
use serde::Serialize;

pub type ClientId = u16;

#[derive(Debug, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct RawTransaction {
    #[serde(rename = "type")]
    transaction_type: TransactionType,
    client: ClientId,
    tx: u32,
    amount: BigDecimal,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(clippy::module_name_repetitions)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug)]
pub enum Transaction {
    Deposit {
        client: ClientId,
        tx: u32,
        amount: BigDecimal,
    },
    Withdrawal {
        client: ClientId,
        tx: u32,
        amount: BigDecimal,
    },
    Dispute {
        client: ClientId,
        tx: u32,
    },
    Resolve {
        client: ClientId,
        tx: u32,
    },
    Chargeback {
        client: ClientId,
        tx: u32,
    },
}

impl Transaction {
    #[allow(clippy::match_same_arms)]
    pub const fn client(&self) -> ClientId {
        *match self {
            Transaction::Deposit { client, .. } => client,
            Transaction::Withdrawal { client, .. } => client,
            Transaction::Dispute { client, .. } => client,
            Transaction::Resolve { client, .. } => client,
            Transaction::Chargeback { client, .. } => client,
        }
    }

    #[allow(clippy::match_same_arms)]
    pub const fn tx(&self) -> u32 {
        *match self {
            Transaction::Deposit { tx, .. } => tx,
            Transaction::Withdrawal { tx, .. } => tx,
            Transaction::Dispute { tx, .. } => tx,
            Transaction::Resolve { tx, .. } => tx,
            Transaction::Chargeback { tx, .. } => tx,
        }
    }
}

impl From<RawTransaction> for Transaction {
    fn from(t: RawTransaction) -> Self {
        match t.transaction_type {
            TransactionType::Deposit => Self::Deposit {
                client: t.client,
                tx: t.tx,
                amount: t.amount,
            },
            TransactionType::Withdrawal => Self::Withdrawal {
                client: t.client,
                tx: t.tx,
                amount: t.amount,
            },
            TransactionType::Dispute => Self::Dispute {
                client: t.client,
                tx: t.tx,
            },
            TransactionType::Resolve => Self::Resolve {
                client: t.client,
                tx: t.tx,
            },
            TransactionType::Chargeback => Self::Chargeback {
                client: t.client,
                tx: t.tx,
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct CsvAccount {
    pub client: ClientId,
    pub available: String,
    pub held: String,
    pub total: String,
    pub locked: bool,
}
