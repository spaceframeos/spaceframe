use std::fmt::{Display, Formatter};

use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use chrono::{DateTime, Local, NaiveDateTime, Utc};

use spaceframe_crypto::ed25519::Ed25519KeyPair;
use spaceframe_crypto::traits::{Keypair, PublicKey};

use crate::account::Address;
use crate::error::TransactionError;

const CONTEXT: &[u8] = b"SpaceframeTxnSigning";

pub type Tx = Transaction<Ed25519KeyPair>;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Clone, Debug)]
pub struct TransactionPayload {
    pub timestamp: i64,
    pub to_address: Address,
    pub amount: u64,
    pub fee: u64,
}

impl TransactionPayload {
    pub fn finalize<T: Keypair>(self, keypair: &T) -> Result<Transaction<T>> {
        let signature = keypair
            .sign(self.as_bytes(), Some(CONTEXT))
            .or(Err(TransactionError::TxSignatureError))?;

        Ok(Transaction {
            signature: Some(TransactionSignature {
                pubkey: keypair.public_key(),
                signature,
            }),
            payload: self,
        })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap()
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Clone, Debug)]
pub struct TransactionSignature<T: PublicKey> {
    pub pubkey: T,
    pub signature: T::SignatureType,
}

impl<T: PublicKey> TransactionSignature<T> {
    pub fn verify<D: AsRef<[u8]>>(&self, data: D) -> Result<()> {
        self.pubkey
            .verify(&self.signature, data, Some(CONTEXT))
            .or(Err(TransactionError::TxInvalidSignature.into()))
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Transaction<T: Keypair> {
    pub signature: Option<TransactionSignature<T::PublicKeyType>>,
    pub payload: TransactionPayload,
}

impl<T: Keypair> Transaction<T> {
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

    pub fn new(keypair: &T, receiver_address: &Address, amount: u64, fee: u64) -> Result<Self> {
        if Address::from(keypair.public_key()) == *receiver_address {
            return Err(TransactionError::TxSelfTransaction.into());
        }

        if amount == 0 {
            return Err(TransactionError::TxInvalidAmount.into());
        }

        let payload = TransactionPayload {
            timestamp: Utc::now().timestamp(),
            to_address: receiver_address.clone(),
            amount,
            fee,
        };
        payload.finalize(&keypair)
    }

    pub fn verify(&self) -> Result<()> {
        self.signature
            .as_ref()
            .map_or(Err(TransactionError::TxNoSignature.into()), |s| {
                s.verify(self.payload.as_bytes())
            })
    }
}

impl Display for Tx {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[from: {}, to: {}, amount: {}, fee: {}, datetime: {}]",
            self.signature
                .as_ref()
                .map_or(String::from("none"), |x| Address::from(x.pubkey)
                    .to_string()),
            self.payload.to_address,
            self.payload.amount,
            self.payload.fee,
            DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(self.payload.timestamp, 0),
                Utc
            )
            .with_timezone(&Local)
        )
    }
}

impl Clone for Tx {
    fn clone(&self) -> Self {
        Transaction {
            signature: self.signature.clone(),
            payload: self.payload.clone(),
        }
    }
}

impl PartialEq for Tx {
    fn eq(&self, other: &Self) -> bool {
        self.payload == other.payload && self.signature == other.signature
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use spaceframe_crypto::ed25519::{Ed25519KeyPair, Ed25519Signature};

    use super::*;

    fn setup() -> (TransactionPayload, Ed25519KeyPair, Ed25519KeyPair) {
        let keypair_1 = Ed25519KeyPair::generate();
        let keypair_2 = Ed25519KeyPair::generate();

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
        let keypair = Ed25519KeyPair::generate();
        let keypair_2 = Ed25519KeyPair::generate();

        let tx = Tx::new(&keypair, &Address::from(keypair_2.public), 13, 2);
        assert!(tx.is_ok());
    }

    #[test]
    fn test_new_transaction_0_amount() {
        let keypair = Ed25519KeyPair::generate();
        let keypair_2 = Ed25519KeyPair::generate();

        let tx = Tx::new(&keypair, &Address::from(keypair_2.public), 0, 0);
        assert!(tx.is_err());
    }

    #[test]
    fn test_new_transaction_self() {
        let keypair = Ed25519KeyPair::generate();
        let tx = Tx::new(&keypair, &Address::from(keypair.public), 12, 1);
        assert!(tx.is_err());
    }

    #[test]
    fn test_verify_no_signature() {
        let keypair = Ed25519KeyPair::generate();
        let tx = Tx::genesis(&Address::from(keypair.public), 1234);
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
        tx.payload.to_address = Ed25519KeyPair::generate().public.into();
        assert!(tx.verify().is_err());
    }

    #[test]
    fn test_txpayload_invalid_signature() {
        let (payload, keypair_1, _) = setup();

        let mut tx = payload.finalize(&keypair_1).unwrap();
        tx.signature.as_mut().unwrap().signature = Ed25519Signature::zero();
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
