use chrono::Utc;
use serde::{Deserialize, Serialize};


use crate::transaction::{Transaction};
use spaceframe_merkle_tree::MerkleTree;
use spaceframe_crypto::hash::Hash;

#[derive(Serialize, Deserialize, Debug)]
pub struct RawBlock {
    height: u64,
    previous_hash: Vec<u8>,
    transactions: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    height: u64,
    timestamp: i64,
    hash: Vec<u8>,
    previous_hash: Vec<u8>,
    merkle_root: Vec<u8>,
    transactions: Vec<Transaction>,
}

impl Block {
    pub fn genesis() -> Self {
        Block {
            height: 0,
            timestamp: Utc::now().naive_utc().timestamp(),
            hash: vec![],
            previous_hash: vec![],
            merkle_root: vec![],
            transactions: vec![],
        }
    }

    pub fn new(raw_block: RawBlock) -> Result<Self, Box<dyn std::error::Error>> {
        let block_bytes = bincode::serialize(&raw_block).unwrap();
        let block_hash = Hash::hash(block_bytes);

        let mut tx_hashes = Vec::new();

        for transaction in raw_block.transactions {
            transaction.verify()?;
            tx_hashes.push(transaction.hash);
        }

        let merkle_tree = MerkleTree::new().with_transactions();

        Block {
            height: raw_block.height,
            timestamp: Utc::now().naive_utc().timestamp(),
            hash: block_hash.to_vec(),
            previous_hash: raw_block.previous_hash,
            merkle_root:,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockData {
    payload: Vec<Transaction>,
}
