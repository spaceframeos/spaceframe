use crate::account::Address;
use crate::errors::LedgerError;
use crate::errors::Result;
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use ed25519_dalek::{Digest, Keypair, PublicKey, Sha512, Signature};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

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

    pub fn finalize(self, keypair: &Keypair) -> Result<Transaction> {
        let signature = keypair
            .sign_prehashed(self.prehashed(), Some(CONTEXT))
            .or(Err(LedgerError::TxSignatureError))?;

        Ok(Transaction {
            signature,
            from_key: keypair.public,
            payload: self,
        })
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

impl Display for Transaction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[from: {}, to: {}, amount: {}, datetime: {}]",
            Address::from(self.from_key),
            self.payload.to_address,
            self.payload.amount,
            DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(self.payload.timestamp, 0),
                Utc
            )
            .with_timezone(&Local)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rand::rngs::OsRng;

    fn setup() -> (TransactionPayload, Keypair, Keypair) {
        let keypair_1: Keypair = Keypair::generate(&mut OsRng);
        let keypair_2: Keypair = Keypair::generate(&mut OsRng);

        (
            TransactionPayload {
                timestamp: Utc::now().timestamp(),
                amount: 12.4,
                to_address: keypair_2.public.into(),
            },
            keypair_1,
            keypair_2,
        )
    }

    #[test]
    fn test_txpayload_verify() {
        let (payload, keypair_1, _) = setup();

        let tx = payload.finalize(&keypair_1).unwrap();
        println!("Transaction: {}", tx);
        assert!(tx.verify().is_ok());
    }

    #[test]
    fn test_txpayload_tampered_1() {
        let (payload, keypair_1, _) = setup();

        let mut tx = payload.finalize(&keypair_1).unwrap();
        tx.payload.amount = 124.4;
        println!("Transaction: {}", tx);
        assert!(tx.verify().is_err());
    }

    #[test]
    fn test_txpayload_tampered_2() {
        let (payload, keypair_1, _) = setup();

        let mut tx = payload.finalize(&keypair_1).unwrap();
        tx.payload.timestamp += 1;
        println!("Transaction: {}", tx);
        assert!(tx.verify().is_err());
    }

    #[test]
    fn test_txpayload_tampered_3() {
        let (payload, keypair_1, _) = setup();

        let mut tx = payload.finalize(&keypair_1).unwrap();
        tx.payload.to_address = Keypair::generate(&mut OsRng).public.into();
        println!("Transaction: {}", tx);
        assert!(tx.verify().is_err());
    }

    #[test]
    fn test_txpayload_invalid_signature() {
        let (payload, keypair_1, _) = setup();

        let mut tx = payload.finalize(&keypair_1).unwrap();
        tx.signature = Signature::new([0u8; 64]);
        println!("Transaction: {}", tx);
        assert!(tx.verify().is_err());
    }

    #[test]
    fn test_txpayload_invalid_pubkey() {
        let (payload, keypair_1, keypair_2) = setup();

        let mut tx = payload.finalize(&keypair_1).unwrap();
        tx.from_key = keypair_2.public;
        println!("Transaction: {}", tx);
        assert!(tx.verify().is_err());
    }
}
