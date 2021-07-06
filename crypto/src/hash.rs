use hex::encode;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hash {
    hash: [u8; Hash::LENGTH],
}

impl Hash {
    pub const LENGTH: usize = 32;

    pub fn new(hash: [u8; Hash::LENGTH]) -> Self {
        Hash { hash }
    }

    pub fn hash<T: AsRef<[u8]>>(value: T) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(value.as_ref());
        Hash {
            hash: hasher.finalize().as_bytes().to_owned(),
        }
    }

    pub fn concat_and_hash(values: &[Self]) -> Self {
        let mut hasher = blake3::Hasher::new();
        for val in values {
            hasher.update(val.as_ref());
        }
        Hash {
            hash: hasher.finalize().as_bytes().to_owned(),
        }
    }

    pub const fn zero() -> Self {
        Hash {
            hash: [0; Hash::LENGTH],
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.hash.to_vec()
    }

    pub fn to_hex(&self) -> String {
        encode(self.as_ref())
    }
}

impl AsRef<[u8; Hash::LENGTH]> for Hash {
    fn as_ref(&self) -> &[u8; Hash::LENGTH] {
        &self.hash
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", self.to_hex())
    }
}
