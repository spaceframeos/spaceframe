use crate::account::Address;
use crate::block::Block;
use crate::errors::{BlockError, LedgerError};
use crate::transaction::Tx;
use anyhow::Result;
use std::collections::HashMap;

#[derive(PartialEq, Debug)]
pub struct Ledger {
    blockchain: Vec<Block>,
}

impl Ledger {
    pub fn new(initial_transactions: &[Tx]) -> Result<Self> {
        let mut blockchain = Vec::new();
        blockchain.push(Block::genesis(initial_transactions)?);
        Ok(Ledger { blockchain })
    }

    // pub fn verify(&self) -> Result<()> {
    //     for i in 0..self.blockchain.len() {
    //         if i > 0 {
    //             // Check height
    //             if self.blockchain[i].height != self.blockchain[i - 1].height + 1 {
    //                 return Err(LedgerError::ChainInvalidHeights);
    //             }
    //
    //             // Check previous hash
    //             if self.blockchain[i]
    //                 .previous_block_hash
    //                 .as_deref()
    //                 .ok_or(LedgerError::ChainPreviousHashMissing)?
    //                 != self.blockchain[i - 1].hash.as_slice()
    //             {
    //                 return Err(LedgerError::ChainInvalidHashes);
    //             }
    //         }
    //
    //         self.blockchain[i].verify()?;
    //
    //     }
    //
    //     Ok(())
    // }

    pub fn add_block(&mut self, block: Block) -> Result<()> {
        let next_height = self.get_current_height() + 1;
        if block.height != next_height {
            return Err(BlockError::BlockInvalidHeight.into());
        }
        if block.previous_block_hash.is_none()
            || block.previous_block_hash.as_deref().unwrap() != self.blockchain.last().unwrap().hash
        {
            return Err(LedgerError::ChainInvalidHashes.into());
        }

        self.check_transactions_balance(&block)?;

        self.blockchain.push(block);
        Ok(())
    }

    pub fn add_block_from_transactions(&mut self, transactions: &[Tx]) -> Result<()> {
        let next_height = self.get_current_height() + 1;
        let previous_hash = self
            .blockchain
            .last()
            .ok_or(LedgerError::ChainNoGenesis)?
            .hash
            .as_slice();

        if previous_hash.is_empty() {
            return Err(LedgerError::ChainInvalidHashes.into());
        }

        let blk = Block::new(next_height, transactions, previous_hash)?;
        self.check_transactions_balance(&blk)?;

        self.blockchain.push(blk);
        Ok(())
    }

    fn check_transactions_balance(&self, block: &Block) -> Result<()> {
        // Get balances for addresses in the transactions
        let mut balances = HashMap::new();
        for tx in &block.transactions {
            let from_addr = Address::from(tx.signature.as_ref().unwrap().pubkey);
            let to_addr = &tx.payload.to_address;
            if !balances.contains_key(&from_addr) {
                balances.insert(from_addr.clone(), self.get_balance(&from_addr)? as i128);
            }
            if !balances.contains_key(to_addr) {
                balances.insert(to_addr.clone(), self.get_balance(to_addr)? as i128);
            }
            *balances.get_mut(&from_addr).unwrap() -= (tx.payload.amount + tx.payload.fee) as i128;
            *balances.get_mut(to_addr).unwrap() += tx.payload.amount as i128;
        }

        if balances.iter().any(|a| a.1 < &0i128) {
            return Err(LedgerError::LedgerBalanceError(None).into());
        }

        Ok(())
    }

    pub fn get_balance(&self, address: &Address) -> Result<u64> {
        let mut balance = 0u64;
        for blk in &self.blockchain {
            let income: u64 = blk
                .transactions
                .iter()
                .map(|t| {
                    return if &t.payload.to_address == address {
                        t.payload.amount
                    } else {
                        0
                    };
                })
                .sum();
            let outcome: u64 = blk
                .transactions
                .iter()
                .map(|t| {
                    return if t.signature.is_some()
                        && &Address::from(t.signature.as_ref().unwrap().pubkey) == address
                    {
                        t.payload.amount + t.payload.fee
                    } else {
                        0
                    };
                })
                .sum();
            balance += income;
            if balance >= outcome {
                balance -= outcome;
            } else {
                return Err(LedgerError::LedgerBalanceError(Some(*address)).into());
            }
        }

        Ok(balance)
    }

    fn get_current_height(&self) -> u64 {
        self.blockchain.last().map_or(0, |b| b.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::Address;
    use spaceframe_crypto::ed25519::Ed25519KeyPair;
    use spaceframe_crypto::traits::Keypair;

    #[test]
    fn test_new_ledger() {
        let ledger = Ledger::new(&[]).unwrap();
        assert_eq!(1, ledger.blockchain.len());
        assert!(ledger.blockchain[0].is_genesis());
    }

    #[test]
    fn test_new_with_transactions() {
        let keypair_2 = Ed25519KeyPair::generate();
        let ledger = Ledger::new(&[
            Tx::genesis(&Address::from(keypair_2.public), 13),
            Tx::genesis(&Address::from(keypair_2.public), 15),
            Tx::genesis(&Address::from(keypair_2.public), 12),
        ]);
        assert!(ledger.is_ok());
    }

    #[test]
    fn test_new_with_invalid_transactions() {
        let keypair = Ed25519KeyPair::generate();
        let keypair_2 = Ed25519KeyPair::generate();
        let ledger = Ledger::new(&[
            Tx::new(&keypair, &Address::from(keypair_2.public), 13, 2).unwrap(),
            Tx::new(&keypair, &Address::from(keypair_2.public), 15, 3).unwrap(),
            Tx::new(&keypair, &Address::from(keypair_2.public), 12, 1).unwrap(),
        ]);
        assert!(ledger.is_err());
    }

    #[test]
    fn test_add_empty_block() {
        let keypair_2 = Ed25519KeyPair::generate();
        let mut ledger = Ledger::new(&[
            Tx::genesis(&Address::from(keypair_2.public), 13),
            Tx::genesis(&Address::from(keypair_2.public), 15),
            Tx::genesis(&Address::from(keypair_2.public), 12),
        ])
        .unwrap();
        let res = ledger.add_block_from_transactions(&[]);
        assert!(res.is_ok());
        assert_eq!(2, ledger.blockchain.len());
    }

    #[test]
    fn test_balance_genesis() {
        let user1 = Ed25519KeyPair::generate();
        let ledger = Ledger::new(&[
            Tx::genesis(&Address::from(user1.public), 13),
            Tx::genesis(&Address::from(user1.public), 15),
            Tx::genesis(&Address::from(user1.public), 12),
        ])
        .unwrap();

        assert_eq!(
            13 + 15 + 12,
            ledger.get_balance(&Address::from(user1.public)).unwrap()
        );
    }

    #[test]
    fn test_balance_with_transactions() {
        let user1 = Ed25519KeyPair::generate();
        let user2 = Ed25519KeyPair::generate();
        let mut ledger = Ledger::new(&[
            Tx::genesis(&Address::from(user1.public), 13),
            Tx::genesis(&Address::from(user2.public), 15),
        ])
        .unwrap();

        ledger
            .add_block_from_transactions(&[
                Tx::new(&user1, &Address::from(user2.public), 5, 2).unwrap()
            ])
            .unwrap();

        assert_eq!(
            13 - (5 + 2),
            ledger.get_balance(&Address::from(user1.public)).unwrap()
        );
        assert_eq!(
            15 + 5,
            ledger.get_balance(&Address::from(user2.public)).unwrap()
        );
    }

    #[test]
    fn test_balance_with_more_transactions() {
        let user1 = Ed25519KeyPair::generate();
        let user2 = Ed25519KeyPair::generate();
        let mut ledger = Ledger::new(&[
            Tx::genesis(&Address::from(user1.public), 13),
            Tx::genesis(&Address::from(user2.public), 15),
        ])
        .unwrap();

        ledger
            .add_block_from_transactions(&[
                Tx::new(&user1, &Address::from(user2.public), 5, 2).unwrap()
            ])
            .unwrap();

        ledger
            .add_block_from_transactions(&[
                Tx::new(&user2, &Address::from(user1.public), 3, 1).unwrap(),
                Tx::new(&user2, &Address::from(user1.public), 5, 2).unwrap(),
                Tx::new(&user1, &Address::from(user2.public), 6, 3).unwrap(),
            ])
            .unwrap();

        assert_eq!(
            13 - (5 + 2) + 3 + 5 - (6 + 3),
            ledger.get_balance(&Address::from(user1.public)).unwrap()
        );
        assert_eq!(
            15 + 5 - (3 + 1) - (5 + 2) + 6,
            ledger.get_balance(&Address::from(user2.public)).unwrap()
        );
    }

    #[test]
    fn test_add_block_invalid_balance() {
        let user1 = Ed25519KeyPair::generate();
        let user2 = Ed25519KeyPair::generate();
        let mut ledger = Ledger::new(&[
            Tx::genesis(&Address::from(user1.public), 13),
            Tx::genesis(&Address::from(user2.public), 15),
        ])
        .unwrap();

        let res = ledger.add_block_from_transactions(&[Tx::new(
            &user1,
            &Address::from(user2.public),
            14,
            2,
        )
        .unwrap()]);

        assert!(res.is_err());
        assert_eq!(1, ledger.blockchain.len());

        let res = ledger.add_block_from_transactions(&[Tx::new(
            &user2,
            &Address::from(user1.public),
            15,
            1,
        )
        .unwrap()]);

        assert!(res.is_err());
        assert_eq!(1, ledger.blockchain.len());

        let res = ledger.add_block_from_transactions(&[
            Tx::new(&user2, &Address::from(user1.public), 7, 1).unwrap(),
            Tx::new(&user2, &Address::from(user1.public), 7, 1).unwrap(),
        ]);

        assert!(res.is_err());
        assert_eq!(1, ledger.blockchain.len());

        let res = ledger.add_block_from_transactions(&[Tx::new(
            &user2,
            &Address::from(user1.public),
            14,
            1,
        )
        .unwrap()]);

        assert!(res.is_ok());
        assert_eq!(2, ledger.blockchain.len());
        assert_eq!(0, ledger.get_balance(&Address::from(user2.public)).unwrap());
    }
}
