use serde::{Deserialize, Serialize};

use crate::account::Address;
use crate::block::Block;
use crate::errors::{LedgerError, Result};
use crate::transaction::Transaction;

#[derive(Serialize, Deserialize, Debug)]
pub struct Ledger {
    blockchain: Vec<Block>,
}

impl Ledger {
    pub fn new(initial_transactions: &[Transaction]) -> Result<Self> {
        let mut blockchain = Vec::new();
        blockchain.push(Block::genesis(initial_transactions)?);
        Ok(Ledger { blockchain })
    }

    pub fn verify(&self) -> Result<()> {
        for i in 0..self.blockchain.len() {
            if i > 0 {
                // Check height
                if self.blockchain[i].height != self.blockchain[i - 1].height + 1 {
                    return Err(LedgerError::ChainInvalidHeights);
                }

                // Check previous hash
                if self.blockchain[i]
                    .previous_block_hash
                    .as_deref()
                    .ok_or(LedgerError::ChainPreviousHashMissing)?
                    != self.blockchain[i - 1].hash.as_slice()
                {
                    return Err(LedgerError::ChainInvalidHashes);
                }
            }

            self.blockchain[i].verify()?;

            // TODO Check transactions in the context of the ledger
        }

        Ok(())
    }

    pub fn add_block(&mut self, transactions: &[Transaction]) -> Result<()> {
        let next_height = self.get_current_height() + 1;
        let previous_hash = self
            .blockchain
            .last()
            .ok_or(LedgerError::ChainNoGenesis)?
            .hash
            .as_slice();

        if previous_hash.is_empty() {
            return Err(LedgerError::ChainInvalidHashes);
        }

        // TODO Check transactions in the context of the ledger

        let blk = Block::new(next_height, transactions, previous_hash)?;
        self.blockchain.push(blk);
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
                return Err(LedgerError::LedgerBalanceError);
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
    use ed25519_dalek::Keypair;
    use rand::rngs::OsRng;

    #[test]
    fn test_new_ledger() {
        let ledger = Ledger::new(&[]).unwrap();
        assert_eq!(1, ledger.blockchain.len());
        assert!(ledger.blockchain[0].is_genesis());
    }

    #[test]
    fn test_new_with_transactions() {
        let keypair: Keypair = Keypair::generate(&mut OsRng);
        let keypair_2: Keypair = Keypair::generate(&mut OsRng);
        let mut ledger = Ledger::new(&[
            Transaction::genesis(&Address::from(keypair_2.public), 13),
            Transaction::genesis(&Address::from(keypair_2.public), 15),
            Transaction::genesis(&Address::from(keypair_2.public), 12),
        ]);
        assert!(ledger.is_ok());
    }

    #[test]
    fn test_new_with_invalid_transactions() {
        let keypair: Keypair = Keypair::generate(&mut OsRng);
        let keypair_2: Keypair = Keypair::generate(&mut OsRng);
        let mut ledger = Ledger::new(&[
            Transaction::new(&keypair, &Address::from(keypair_2.public), 13, 2).unwrap(),
            Transaction::new(&keypair, &Address::from(keypair_2.public), 15, 3).unwrap(),
            Transaction::new(&keypair, &Address::from(keypair_2.public), 12, 1).unwrap(),
        ]);
        assert!(ledger.is_err());
    }

    #[test]
    fn test_verify_empty() {
        let ledger = Ledger::new(&[]).unwrap();
        assert!(ledger.verify().is_ok());
    }

    #[test]
    fn test_add_empty_block() {
        let keypair: Keypair = Keypair::generate(&mut OsRng);
        let keypair_2: Keypair = Keypair::generate(&mut OsRng);
        let mut ledger = Ledger::new(&[
            Transaction::genesis(&Address::from(keypair_2.public), 13),
            Transaction::genesis(&Address::from(keypair_2.public), 15),
            Transaction::genesis(&Address::from(keypair_2.public), 12),
        ])
        .unwrap();
        let res = ledger.add_block(&[]);
        assert!(res.is_ok());
        assert_eq!(2, ledger.blockchain.len());
    }

    #[test]
    fn test_balance_genesis() {
        let user1: Keypair = Keypair::generate(&mut OsRng);
        let mut ledger = Ledger::new(&[
            Transaction::genesis(&Address::from(user1.public), 13),
            Transaction::genesis(&Address::from(user1.public), 15),
            Transaction::genesis(&Address::from(user1.public), 12),
        ])
        .unwrap();

        assert_eq!(
            13 + 15 + 12,
            ledger.get_balance(&Address::from(user1.public)).unwrap()
        );
    }

    #[test]
    fn test_balance_with_transactions() {
        let user1: Keypair = Keypair::generate(&mut OsRng);
        let user2: Keypair = Keypair::generate(&mut OsRng);
        let mut ledger = Ledger::new(&[
            Transaction::genesis(&Address::from(user1.public), 13),
            Transaction::genesis(&Address::from(user2.public), 15),
        ])
        .unwrap();

        ledger
            .add_block(&[Transaction::new(&user1, &Address::from(user2.public), 5, 2).unwrap()])
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
        let user1: Keypair = Keypair::generate(&mut OsRng);
        let user2: Keypair = Keypair::generate(&mut OsRng);
        let mut ledger = Ledger::new(&[
            Transaction::genesis(&Address::from(user1.public), 13),
            Transaction::genesis(&Address::from(user2.public), 15),
        ])
        .unwrap();

        ledger
            .add_block(&[Transaction::new(&user1, &Address::from(user2.public), 5, 2).unwrap()])
            .unwrap();

        ledger
            .add_block(&[
                Transaction::new(&user2, &Address::from(user1.public), 3, 1).unwrap(),
                Transaction::new(&user2, &Address::from(user1.public), 5, 2).unwrap(),
                Transaction::new(&user1, &Address::from(user2.public), 6, 3).unwrap(),
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
        let user1: Keypair = Keypair::generate(&mut OsRng);
        let user2: Keypair = Keypair::generate(&mut OsRng);
        let mut ledger = Ledger::new(&[
            Transaction::genesis(&Address::from(user1.public), 13),
            Transaction::genesis(&Address::from(user2.public), 15),
        ])
        .unwrap();

        let res =
            ledger.add_block(&[
                Transaction::new(&user1, &Address::from(user2.public), 14, 2).unwrap(),
            ]);

        assert!(res.is_err());

        let res =
            ledger.add_block(&[
                Transaction::new(&user2, &Address::from(user1.public), 15, 1).unwrap(),
            ]);

        assert!(res.is_err());
    }
}
