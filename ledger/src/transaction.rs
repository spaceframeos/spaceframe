use crate::account::Address;
use crate::errors::LedgerError;
use crate::errors::Result;
use ed25519_dalek::{Digest, Keypair, PublicKey, Sha512, Signature};
use serde::{Deserialize, Serialize};

const CONTEXT: &[u8] = b"SpaceframeTxnSigning";

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionPayload {
    timestamp: i64,
    to_address: Address,
    amount: f64,
}

impl TransactionPayload {
    pub fn as_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn prehashed(&self) -> Sha512 {
        let mut hasher = Sha512::new();
        hasher.update(self.as_bytes());
        hasher
    }

    pub fn finalize(self, keypair: &Keypair) -> Transaction {
        let signature = keypair
            .sign_prehashed(self.prehashed(), Some(CONTEXT))
            .unwrap();

        Transaction {
            signature,
            from_key: keypair.public,
            payload: self,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub from_key: PublicKey,
    pub signature: Signature,
    pub payload: TransactionPayload,
}

impl Transaction {
    pub fn verify(&self) -> Result<()> {
        self.from_key
            .verify_prehashed(self.payload.prehashed(), Some(CONTEXT), &self.signature)
            .or(Err(LedgerError::TxInvalidSignature))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_txpayload_finalize() {
        // let payload = TransactionPayload {
        //     timestamp: 1234,
        //     amount: 12.4,
        //     to_address:
        // }
    }
}
