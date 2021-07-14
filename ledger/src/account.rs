use base58::ToBase58;
use ed25519_dalek::PublicKey;
use spaceframe_crypto::hash::Hash;
use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

pub struct Account {}

const VERSION: &[u8] = b"01";

#[derive(Serialize, Deserialize, Debug)]
pub struct Address([u8; Address::ADDRESS_LENGTH]);

impl Address {
    pub const ADDRESS_LENGTH: usize = 22;

    pub fn to_hex(&self) -> String {
        self.0.to_base58()
    }
}

impl From<PublicKey> for Address {
    fn from(key: PublicKey) -> Self {
        let address_bytes = key.as_bytes();
        let address_payload = &address_bytes[(address_bytes.len() - 16)..];
        let mut payload = Vec::new();
        payload.extend_from_slice(VERSION); // 2 bytes
        payload.extend_from_slice(address_payload); // 16 bytes
        let checksum = &Hash::hash(&payload).to_vec()[..4];
        payload.extend_from_slice(checksum); // 4 bytes

        Address(<[u8; Address::ADDRESS_LENGTH]>::try_from(payload).unwrap())
    }
}
