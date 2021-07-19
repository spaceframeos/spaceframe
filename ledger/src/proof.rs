use borsh::{BorshDeserialize, BorshSerialize};
use std::fmt::Debug;

pub trait Proof: BorshSerialize + BorshDeserialize + Clone + PartialEq + Debug {}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct PoWorkProof {}

impl Proof for PoWorkProof {}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct PoSpaceProof {}

impl Proof for PoSpaceProof {}
