use std::io;

use anyhow::Result;
use csv::Trim;

use crate::transaction::{RawTransaction, Transaction};
use crate::transaction_engine::TransactionEngine;

mod transaction;
mod transaction_engine;

fn main() -> Result<()> {
    // Hacky argument parsing, for a real CLI I would've used a crate like clap
    let mut args = std::env::args();
    let _binary_path = args.next();
    let file_path = match args.next() {
        None => {
            eprintln!("Expected a filename");
            std::process::exit(-1);
        }
        Some(path) => path,
    };

    let mut transaction_engine = TransactionEngine::new();

    let mut csv_reader = csv::ReaderBuilder::new()
        .trim(Trim::All)
        .from_path(file_path)?;

    for result in csv_reader.deserialize() {
        // Transaction is how I want transactions to be represented,
        // But I couldn't figure out how to use the csv crate to parse directly into that format
        // so I parse into an intermediate, RawTransaction, and then convert manually
        let raw: RawTransaction = result?;
        let transaction: Transaction = raw.into();
        transaction_engine.process(transaction);
    }

    let accounts = transaction_engine.accounts();
    let mut csv_writer = csv::Writer::from_writer(io::stdout());
    for account in accounts {
        csv_writer.serialize(account)?;
    }
    csv_writer.flush()?;

    Ok(())
}
