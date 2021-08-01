use borsh::{BorshDeserialize, BorshSerialize};
use digest::Digest;
use sha2::Sha256;
use std::fmt::Debug;

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct PoWorkProof<const N: u8> {
    nonce: u64,
}

impl<const N: u8> PoWorkProof<N> {
    fn _find_proof<T: AsRef<[u8]>>(data: T) -> Self {
        let difficulty = N;
        let diff_str = (0..difficulty).map(|_| "0").collect::<String>();

        let mut proof: u64 = 0u64;

        loop {
            let mut bytes = proof.to_le_bytes().to_vec();
            bytes.extend_from_slice(data.as_ref());
            let hash = hex::encode(Sha256::digest(&bytes));
            if hash.starts_with(&diff_str) {
                return Self { nonce: proof };
            }
            proof += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::Sha256;

    #[test]
    fn test_powork() {
        let data = b"coucou";
        let pow = PoWorkProof::<3>::_find_proof(&data);
        let mut bytes = pow.nonce.to_le_bytes().to_vec();
        bytes.extend_from_slice(data.as_ref());
        let hash = Sha256::digest(&bytes).to_vec();
        assert!(hex::encode(&hash).starts_with("000"));
    }
}
