use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::errors::Result;
use crate::transaction::Transaction;
use spaceframe_crypto::hash::Hash;
use spaceframe_merkle_tree::MerkleTree;

#[derive(Serialize, Deserialize, Debug)]
pub struct RawBlock {
    height: u64,
    previous_hash: Vec<u8>,
    transactions: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub height: u64,
    pub timestamp: i64,
    pub hash: Vec<u8>,
    pub previous_hash: Option<Vec<u8>>,
    pub merkle_root: Option<Vec<u8>>,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn genesis() -> Self {
        Block {
            height: 0,
            timestamp: Utc::now().naive_utc().timestamp(),
            hash: Hash::zero().to_vec(),
            previous_hash: None,
            merkle_root: None,
            transactions: vec![],
        }
    }

    pub fn new(raw_block: RawBlock) -> Result<Self> {
        let block_bytes = bincode::serialize(&raw_block).unwrap();
        let block_hash = Hash::hash(block_bytes);

        let mut tx_hashes = Vec::new();

        for transaction in &raw_block.transactions {
            transaction.verify()?;
            tx_hashes.push(transaction.payload.as_bytes());
        }

        let merkle_tree = MerkleTree::new()
            .with_transactions(&tx_hashes)
            .root()
            .map(|r| r.to_vec());

        Ok(Block {
            height: raw_block.height,
            timestamp: Utc::now().naive_utc().timestamp(),
            hash: block_hash.to_vec(),
            previous_hash: Some(raw_block.previous_hash),
            merkle_root: merkle_tree,
            transactions: raw_block.transactions,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockData {
    payload: Vec<Transaction>,
}
