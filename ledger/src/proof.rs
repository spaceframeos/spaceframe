use borsh::{BorshDeserialize, BorshSerialize};
use std::fmt::Debug;

pub trait Proof: BorshSerialize + BorshDeserialize + Clone + PartialEq + Debug {
    fn find_proof<T: AsRef<[u8]>>(data: T) -> Self;
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct PoSpaceProof {}

impl Proof for PoSpaceProof {
    fn find_proof<T: AsRef<[u8]>>(_data: T) -> Self {
        todo!()
    }
}
