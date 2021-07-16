use crate::errors::{BlockError, TransactionError};
use crate::transaction::Tx;
use anyhow::Result;
use spaceframe_crypto::hash::Hash;
use spaceframe_merkletree::MerkleTree;

#[derive(PartialEq, Debug)]
pub struct Block {
    pub height: u64,
    pub hash: Vec<u8>,
    pub previous_block_hash: Option<Vec<u8>>,
    pub transactions: Vec<Tx>,
    pub merkle_root: Option<Vec<u8>>,
}

impl Block {
    pub fn genesis(initial_transactions: &[Tx]) -> Result<Self> {
        for tx in initial_transactions {
            if tx.signature.is_some() {
                return Err(TransactionError::GenesisSigned.into());
            }
        }

        let mut blk = Block {
            height: 1,
            hash: Hash::zero().to_vec(),
            transactions: initial_transactions.to_vec(),
            previous_block_hash: None,
            merkle_root: None,
        };
        let hashes = blk.calculate_hash()?;
        blk.hash = hashes.block_hash.to_vec();
        blk.merkle_root = hashes.merkle_root.map(|x| x.to_vec());

        blk.verify()?;

        Ok(blk)
    }

    pub fn new(height: u64, transactions: &[Tx], previous_block_hash: &[u8]) -> Result<Self> {
        // Check height
        if height < 2 {
            return Err(BlockError::BlockInvalidHeight.into());
        }

        // Check transactions
        for tx in transactions {
            tx.verify()?;
        }

        let mut block = Block {
            height,
            transactions: transactions.to_vec(),
            previous_block_hash: Some(previous_block_hash.to_vec()),
            merkle_root: None,
            hash: Vec::new(),
        };

        let block_hash = block.calculate_hash()?;
        block.hash = block_hash.block_hash.to_vec();
        block.merkle_root = block_hash.merkle_root.map(|x| x.to_vec());

        Ok(block)
    }

    pub fn verify(&self) -> Result<()> {
        if (self.merkle_root.is_none() && self.transactions.len() > 0)
            || (self.merkle_root.is_some() && self.transactions.len() == 0)
        {
            return Err(BlockError::BlockInvalid.into());
        }

        let hash = self.calculate_hash()?;

        // Check hash
        if hash.block_hash.to_vec() != self.hash {
            return Err(BlockError::BlockInvalid.into());
        }

        // Check merkle root
        if self.merkle_root.is_some() && hash.merkle_root.is_some() {
            if hash.merkle_root.unwrap().to_vec() != self.merkle_root.as_deref().unwrap() {
                return Err(BlockError::BlockInvalid.into());
            }
        }

        // Verify transactions
        if !self.is_genesis() {
            for tx in &self.transactions {
                tx.verify()?;
            }
        }

        Ok(())
    }

    pub fn is_genesis(&self) -> bool {
        self.previous_block_hash.is_none() && self.height == 1
    }

    fn calculate_hash(&self) -> Result<BlockHash> {
        let mut bytes = self.height.to_be_bytes().to_vec();

        if self.previous_block_hash.is_some() {
            bytes.extend_from_slice(&self.previous_block_hash.as_deref().unwrap());
        }

        let merkle_root = if self.transactions.len() > 0 {
            let root = Self::calculate_merkle_root(&self.transactions)?;
            bytes.extend_from_slice(root.as_ref());
            Some(root)
        } else {
            None
        };

        Ok(BlockHash {
            block_hash: Hash::hash(bytes),
            merkle_root,
        })
    }

    fn calculate_merkle_root(transactions: &[Tx]) -> Result<Hash> {
        let mut tx_bytes = Vec::new();
        for transaction in transactions {
            tx_bytes.push(transaction.payload.as_bytes());
        }

        let merkle_tree = MerkleTree::new().with_transactions(&tx_bytes);

        Ok(merkle_tree
            .root()
            .ok_or(BlockError::BlockEmptyMerkleRoot)?
            .clone())
    }
}

struct BlockHash {
    block_hash: Hash,
    merkle_root: Option<Hash>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::Address;
    use spaceframe_crypto::ed25519::Ed25519KeyPair;
    use spaceframe_crypto::traits::Keypair;

    #[test]
    fn test_new_genesis_no_transaction() {
        let initial_transactions = Vec::new();
        let genesis = Block::genesis(&initial_transactions).unwrap();
        assert!(genesis.merkle_root.is_none());
        assert!(genesis.previous_block_hash.is_none());
        assert_eq!(initial_transactions.len(), genesis.transactions.len());
    }

    #[test]
    fn test_new_genesis_with_transactions() {
        let keypair = Ed25519KeyPair::generate();
        let initial_transactions = vec![Tx::genesis(&Address::from(keypair.public), 1234)];
        let genesis = Block::genesis(&initial_transactions).unwrap();
        assert!(genesis.merkle_root.is_some());
        assert!(genesis.previous_block_hash.is_none());
        assert_eq!(initial_transactions.len(), genesis.transactions.len());
    }

    #[test]
    fn test_verify_genesis_no_transaction() {
        let genesis = Block::genesis(&[]).unwrap();
        let res = genesis.verify();
        assert!(res.is_ok());
    }

    #[test]
    fn test_verify_genesis_with_transactions() {
        let keypair = Ed25519KeyPair::generate();
        let initial_transactions = vec![Tx::genesis(&Address::from(keypair.public), 1234)];
        let genesis = Block::genesis(&initial_transactions).unwrap();
        let res = genesis.verify();
        assert!(res.is_ok());
    }

    #[test]
    fn test_new_empty() {
        let empty = Block::new(12, &[], &Hash::zero().to_vec());
        assert!(empty.is_ok());
    }

    #[test]
    fn test_new_incorrect_height() {
        let empty = Block::new(1, &[], &Hash::zero().to_vec());
        assert!(empty.is_err());
    }

    #[test]
    fn test_new_with_transactions() {
        let keypair = Ed25519KeyPair::generate();
        let keypair_2 = Ed25519KeyPair::generate();

        let block = Block::new(
            2,
            &[
                Tx::new(&keypair, &Address::from(keypair_2.public), 13, 2).unwrap(),
                Tx::new(&keypair, &Address::from(keypair_2.public), 15, 2).unwrap(),
                Tx::new(&keypair, &Address::from(keypair_2.public), 12, 2).unwrap(),
            ],
            &Hash::zero().to_vec(),
        );
        assert!(block.is_ok());
    }

    #[test]
    fn test_new_with_invalid_transactions() {
        let keypair = Ed25519KeyPair::generate();
        let keypair_2 = Ed25519KeyPair::generate();

        let block = Block::new(
            2,
            &[
                Tx::new(&keypair, &Address::from(keypair_2.public), 13, 2).unwrap(),
                Tx::genesis(&Address::from(keypair_2.public), 15),
                Tx::new(&keypair, &Address::from(keypair_2.public), 12, 2).unwrap(),
            ],
            &Hash::zero().to_vec(),
        );
        assert!(block.is_err());
    }

    #[test]
    fn test_verify_empty_block() {
        let empty = Block::new(12, &[], &Hash::zero().to_vec()).unwrap();
        let res = empty.verify();
        assert!(res.is_ok());
    }

    #[test]
    fn test_verify_with_transactions() {
        let keypair = Ed25519KeyPair::generate();
        let keypair_2 = Ed25519KeyPair::generate();

        let blk = Block::new(
            12,
            &[
                Tx::new(&keypair, &Address::from(keypair_2.public), 13, 2).unwrap(),
                Tx::new(&keypair, &Address::from(keypair_2.public), 15, 2).unwrap(),
                Tx::new(&keypair, &Address::from(keypair_2.public), 12, 2).unwrap(),
            ],
            &Hash::zero().to_vec(),
        )
        .unwrap();

        let res = blk.verify();
        assert!(res.is_ok());
    }

    #[test]
    fn test_verify_invalid_hash() {
        let keypair = Ed25519KeyPair::generate();
        let keypair_2 = Ed25519KeyPair::generate();

        let mut blk = Block::new(
            12,
            &[
                Tx::new(&keypair, &Address::from(keypair_2.public), 13, 2).unwrap(),
                Tx::new(&keypair, &Address::from(keypair_2.public), 15, 2).unwrap(),
                Tx::new(&keypair, &Address::from(keypair_2.public), 12, 2).unwrap(),
            ],
            &Hash::zero().to_vec(),
        )
        .unwrap();

        blk.hash = Hash::zero().to_vec();

        let res = blk.verify();
        assert!(res.is_err());
    }

    #[test]
    fn test_verify_invalid_merkle_root() {
        let keypair = Ed25519KeyPair::generate();
        let keypair_2 = Ed25519KeyPair::generate();

        let mut blk = Block::new(
            12,
            &[
                Tx::new(&keypair, &Address::from(keypair_2.public), 13, 2).unwrap(),
                Tx::new(&keypair, &Address::from(keypair_2.public), 15, 2).unwrap(),
                Tx::new(&keypair, &Address::from(keypair_2.public), 12, 2).unwrap(),
            ],
            &Hash::zero().to_vec(),
        )
        .unwrap();

        blk.merkle_root = Some(Hash::zero().to_vec());

        let res = blk.verify();
        assert!(res.is_err());
    }

    #[test]
    fn test_verify_invalid_transaction() {
        let keypair = Ed25519KeyPair::generate();
        let keypair_2 = Ed25519KeyPair::generate();

        let mut blk = Block::new(
            12,
            &[
                Tx::new(&keypair, &Address::from(keypair_2.public), 13, 2).unwrap(),
                Tx::new(&keypair, &Address::from(keypair_2.public), 15, 2).unwrap(),
                Tx::new(&keypair, &Address::from(keypair_2.public), 12, 2).unwrap(),
            ],
            &Hash::zero().to_vec(),
        )
        .unwrap();

        // Tamper the block
        blk.transactions[1] = Tx::new(&keypair, &Address::from(keypair_2.public), 14, 2).unwrap();

        let res = blk.verify();
        assert!(res.is_err());
    }
}
