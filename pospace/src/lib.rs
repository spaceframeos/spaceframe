use bitvec::prelude::*;

pub mod bits;
pub mod collation;
pub mod constants;
pub mod core;
pub mod f1_calculator;
pub mod fx_calculator;
pub mod proofs;
pub mod sort;
pub mod storage;
pub mod utils;
pub mod verifier;

pub type Bits = BitVec<Lsb0, u8>;
pub type BitsSlice = BitSlice<Lsb0, u8>;

#[macro_use]
extern crate lazy_static;
