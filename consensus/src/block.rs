use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    height: u64,
    hash: Vec<u8>,
    previous_hash: Vec<u8>,
    merkle_root: Vec<u8>,
    transactions: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    hash: Vec<u8>,
    signature: Vec<u8>,
    timestamp: u64,
    inputs: Vec<TransactionIO>,
    outputs: Vec<TransactionIO>,
    fee: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionIO {
    address: Vec<u8>,
    amount: f64,
}
