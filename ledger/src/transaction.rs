use crate::account::Address;
use crate::errors::LedgerError;
use crate::errors::Result;
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use ed25519_dalek::{Digest, Keypair, PublicKey, Sha512, Signature};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

const CONTEXT: &[u8] = b"SpaceframeTxnSigning";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionPayload {
    pub timestamp: i64,
    pub to_address: Address,
    pub amount: u64,
    pub fee: u64,
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
            signature: Some(TransactionSignature {
                pubkey: keypair.public,
                signature,
            }),
            payload: self,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionSignature {
    pub pubkey: PublicKey,
    pub signature: Signature,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub signature: Option<TransactionSignature>,
    pub payload: TransactionPayload,
}

impl Transaction {
    pub fn genesis(address: &Address, amount: u64) -> Self {
        Transaction {
            payload: TransactionPayload {
                fee: 0,
                amount,
                to_address: address.clone(),
                timestamp: Utc::now().timestamp(),
            },
            signature: None,
        }
    }

    pub fn new(
        keypair: &Keypair,
        receiver_address: &Address,
        amount: u64,
        fee: u64,
    ) -> Result<Self> {
        if Address::from(keypair.public) == *receiver_address {
            return Err(LedgerError::TxSelfTransaction);
        }

        if amount == 0 {
            return Err(LedgerError::TxInvalidAmount);
        }

        let payload = TransactionPayload {
            timestamp: Utc::now().timestamp(),
            to_address: receiver_address.clone(),
            amount,
            fee,
        };
        payload.finalize(keypair)
    }

    pub fn verify(&self) -> Result<()> {
        self.signature
            .as_ref()
            .map_or(Err(LedgerError::TxNoSignature), |s| {
                s.pubkey
                    .verify_prehashed(self.payload.prehashed(), Some(CONTEXT), &s.signature)
                    .or(Err(LedgerError::TxInvalidSignature))
            })
    }

}

impl Display for Transaction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[from: {}, to: {}, amount: {}, datetime: {}]",
            self.signature
                .as_ref()
                .map_or(String::from("none"), |x| Address::from(x.pubkey)
                    .to_string()),
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
                to_address: keypair_2.public.into(),
                amount: 12,
                fee: 1,
            },
            keypair_1,
            keypair_2,
        )
    }

    #[test]
    fn test_new_transaction() {
        let keypair: Keypair = Keypair::generate(&mut OsRng);
        let keypair_2: Keypair = Keypair::generate(&mut OsRng);

        let tx = Transaction::new(&keypair, &Address::from(keypair_2.public), 13, 2);
        assert!(tx.is_ok());
    }

    #[test]
    fn test_new_transaction_0_amount() {
        let keypair: Keypair = Keypair::generate(&mut OsRng);
        let keypair_2: Keypair = Keypair::generate(&mut OsRng);

        let tx = Transaction::new(&keypair, &Address::from(keypair_2.public), 0, 0);
        assert!(tx.is_err());
    }

    #[test]
    fn test_new_transaction_self() {
        let keypair: Keypair = Keypair::generate(&mut OsRng);
        let tx = Transaction::new(&keypair, &Address::from(keypair.public), 12, 1);
        assert!(tx.is_err());
    }

    #[test]
    fn test_verify_no_signature() {
        let keypair: Keypair = Keypair::generate(&mut OsRng);
        let tx = Transaction::genesis(&Address::from(keypair.public), 1234);
        assert!(tx.verify().is_err());
    }

    #[test]
    fn test_txpayload_verify() {
        let (payload, keypair_1, _) = setup();

        let tx = payload.finalize(&keypair_1).unwrap();
        assert!(tx.verify().is_ok());
    }

    #[test]
    fn test_txpayload_tampered_1() {
        let (payload, keypair_1, _) = setup();

        let mut tx = payload.finalize(&keypair_1).unwrap();
        tx.payload.amount = 124;
        assert!(tx.verify().is_err());
    }

    #[test]
    fn test_txpayload_tampered_2() {
        let (payload, keypair_1, _) = setup();

        let mut tx = payload.finalize(&keypair_1).unwrap();
        tx.payload.timestamp += 1;
        assert!(tx.verify().is_err());
    }

    #[test]
    fn test_txpayload_tampered_3() {
        let (payload, keypair_1, _) = setup();

        let mut tx = payload.finalize(&keypair_1).unwrap();
        tx.payload.to_address = Keypair::generate(&mut OsRng).public.into();
        assert!(tx.verify().is_err());
    }

    #[test]
    fn test_txpayload_invalid_signature() {
        let (payload, keypair_1, _) = setup();

        let mut tx = payload.finalize(&keypair_1).unwrap();
        tx.signature.as_mut().unwrap().signature = Signature::new([0u8; 64]);
        assert!(tx.verify().is_err());
    }

    #[test]
    fn test_txpayload_invalid_pubkey() {
        let (payload, keypair_1, keypair_2) = setup();

        let mut tx = payload.finalize(&keypair_1).unwrap();
        tx.signature.as_mut().unwrap().pubkey = keypair_2.public;
        assert!(tx.verify().is_err());
    }
}
