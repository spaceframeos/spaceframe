use serde::{Deserialize, Serialize};

use crate::transaction::Transaction;

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    height: u64,
    hash: Vec<u8>,
    previous_hash: Vec<u8>,
    merkle_root: Vec<u8>,
    transactions: Vec<Transaction>,
}
