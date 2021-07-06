use serde::{Deserialize, Serialize};

use crate::transaction::Transaction;

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    transactions: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockHeader {
    height: u64,
    timestamp: u64,
    hash: Vec<u8>,
    previous_hash: Vec<u8>,
    merkle_root: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockData {
    payload: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenesisBlock {}
