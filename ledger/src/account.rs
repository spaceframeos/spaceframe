use base58::ToBase58;
use ed25519_dalek::PublicKey;
use spaceframe_crypto::hash::Hash;
use std::convert::TryFrom;

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub struct Account {}

const VERSION: &[u8] = b"01";

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Address([u8; Address::ADDRESS_LENGTH]);

impl Address {
    pub const ADDRESS_LENGTH: usize = 22;

    pub fn to_string(&self) -> String {
        format!("SF_{}", self.0.to_base58())
    }
}

impl FromStr for Address {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Keypair;
    use rand::rngs::OsRng;

    #[test]
    fn test_from_pubkey() {
        let keypair: Keypair = Keypair::generate(&mut OsRng);
        let address: Address = keypair.public.into();
        assert_eq!(address.0.len(), Address::ADDRESS_LENGTH);
    }

    #[test]
    fn test_to_string() {
        let keypair: Keypair = Keypair::generate(&mut OsRng);
        let address: Address = keypair.public.into();
        println!("{}", address.to_string());
    }
}
