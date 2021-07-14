use crate::block::Block;
use crate::errors::Result;

use crate::transaction::Transaction;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Ledger {
    blockchain: Vec<Block>,
}

impl Ledger {
    pub fn new() -> Self {
        let mut blockchain = Vec::new();
        blockchain.push(Block::genesis());
        Ledger { blockchain }
    }

    pub fn verify(&self) -> Result<()> {
        todo!()
    }

    pub fn add_block(&mut self, transactions: &[Transaction]) -> Result<()> {
        todo!()
    }

    // TODO: Change address type
    pub fn get_balance(&self, address: Vec<u8>) -> f64 {
        todo!()
    }

    fn get_current_height(&self) -> usize {
        self.blockchain.len()
    }
}
