use crate::errors::LedgerError;
use crate::errors::Result;
use serde::{Deserialize, Serialize};
use spaceframe_crypto::hash::Hash;

#[derive(Serialize, Deserialize, Debug)]
struct RawTransaction {
    timestamp: i64,
    inputs: Vec<TransactionIO>,
    outputs: Vec<TransactionIO>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub hash: Vec<u8>,
    pub signature: Vec<u8>,
    pub timestamp: i64,
    pub inputs: Vec<TransactionIO>,
    pub outputs: Vec<TransactionIO>,
}

impl Transaction {
    pub fn verify(&self) -> Result<()> {
        // Recalculate tx hash
        let bytes = bincode::serialize(&RawTransaction {
            timestamp: self.timestamp,
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
        })
        .unwrap();
        let calculated_hash = Hash::hash(bytes);

        // Verify hash
        if calculated_hash.to_vec() != self.hash {
            return Err(LedgerError::TxInvalidHash);
        }

        // TODO Verify signature
        return Err(LedgerError::TxInvalidSignature);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionIO {
    address: Vec<u8>,
    amount: f64,
}
