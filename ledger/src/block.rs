use serde::{Deserialize, Serialize};

use crate::errors::{LedgerError, Result};
use crate::transaction::Transaction;
use spaceframe_crypto::hash::Hash;
use spaceframe_merkle_tree::MerkleTree;

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub hash: Vec<u8>,
    pub previous_block_hash: Option<Vec<u8>>,
    pub transactions: Vec<Transaction>,
    pub merkle_root: Option<Vec<u8>>,
}

impl Block {
    pub fn genesis() -> Self {
        Block {
            hash: Hash::zero().to_vec(),
            transactions: vec![],
            previous_block_hash: None,
            merkle_root: None,
        }
    }

    pub fn new(transactions: &[Transaction], previous_block_hash: &[u8]) -> Result<Self> {
        let mut tx_bytes = Vec::new();
        for transaction in transactions {
            transaction.verify()?;
            tx_bytes.push(transaction.payload.as_bytes());
        }

        let merkle_tree = MerkleTree::new().with_transactions(&tx_bytes);

        let merkle_root = merkle_tree
            .root()
            .ok_or(LedgerError::BlockEmptyMerkleRoot)?;

        let mut block_bytes = merkle_root.to_vec();
        block_bytes.extend_from_slice(previous_block_hash);

        let block_hash = Hash::hash(block_bytes);

        Ok(Block {
            hash: block_hash.to_vec(),
            previous_block_hash: Some(previous_block_hash.to_vec()),
            merkle_root: Some(merkle_root.to_vec()),
            transactions: transactions.to_vec(),
        })
    }

    pub fn verify(&self) -> Result<()> {
        if self.merkle_root.is_none() && self.transactions.len() > 0 {
            return Err(LedgerError::BlockEmptyMerkleRoot);
        }

        if self.merkle_root.is_none() && self.previous_block_hash.is_none() {
            return Ok(());
        }

        // Check hash
        if self.merkle_root.is_some() && self.previous_block_hash.is_some() {
            let mut block_bytes = self.merkle_root.as_ref().unwrap().clone();
            block_bytes.extend(self.previous_block_hash.as_ref().unwrap());
            let block_hash = Hash::hash(block_bytes).to_vec();

            if block_hash != self.hash {
                return Err(LedgerError::BlockInvalid);
            }
        } else {
            return Err(LedgerError::BlockInvalid);
        }

        // Check transactions
        if self.merkle_root.is_some() && self.transactions.len() > 0 {
            let mut tx_bytes = Vec::new();
            for transaction in &self.transactions {
                transaction.verify()?;
                tx_bytes.push(transaction.payload.as_bytes());
            }

            let merkle_tree = MerkleTree::new().with_transactions(&tx_bytes);

            let merkle_root = merkle_tree
                .root()
                .ok_or(LedgerError::BlockEmptyMerkleRoot)?;

            if merkle_root.to_vec() != self.merkle_root.as_deref().unwrap() {
                return Err(LedgerError::BlockInvalid);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_genesis() {}

    #[test]
    fn test_new_empty() {}

    #[test]
    fn test_new_with_transactions() {}

    #[test]
    fn test_verify_genesis() {}

    #[test]
    fn test_verify_empty_block() {}

    #[test]
    fn test_verify_invalid_hash() {}

    #[test]
    fn test_verify_invalid_merkle_root() {}

    #[test]
    fn test_verify_invalid_transaction() {}
}
