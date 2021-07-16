use spaceframe_crypto::hash::Hash;
use std::convert::TryFrom;

use borsh::{BorshDeserialize, BorshSerialize};

use spaceframe_crypto::traits::PublicKey;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub struct Account {}

const VERSION: &[u8] = b"01";

#[derive(BorshSerialize, BorshDeserialize, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Address([u8; Address::ADDRESS_LENGTH]);

impl Address {
    pub const ADDRESS_LENGTH: usize = 22;

    pub fn to_string(&self) -> String {
        format!("SF_{}", bs58::encode(self.0).into_string())
    }
}

impl FromStr for Address {
    type Err = Box<dyn std::error::Error>;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl<T: PublicKey> From<T> for Address {
    fn from(key: T) -> Self {
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
    use spaceframe_crypto::ed25519::Ed25519KeyPair;
    use spaceframe_crypto::traits::Keypair;

    #[test]
    fn test_from_pubkey() {
        let keypair: Ed25519KeyPair = Ed25519KeyPair::generate();
        let address: Address = keypair.public.into();
        assert_eq!(address.0.len(), Address::ADDRESS_LENGTH);
    }

    #[test]
    fn test_to_string() {
        let keypair: Ed25519KeyPair = Ed25519KeyPair::generate();
        let address: Address = keypair.public.into();
        println!("{}", address.to_string());
    }
}
