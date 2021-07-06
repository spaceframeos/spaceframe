use blake3::Hash as BLK3Hash;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hash {
    hash: BLK3Hash,
}

impl Hash {
    pub fn concat(hashes: &[Self]) -> Self {
        let mut hasher = blake3::Hasher::new();
        for hash in hashes {
            hasher.update(hash.bytes());
        }
        Hash {
            hash: hasher.finalize(),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        self.hash.as_bytes()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.bytes().to_vec()
    }

    pub fn hex(&self) -> String {
        self.hash.to_hex().to_string()
    }
}

impl<T: AsRef<[u8]>> From<T> for Hash {
    fn from(data: T) -> Self {
        Hash {
            hash: blake3::hash(data.as_ref()),
        }
    }
}
