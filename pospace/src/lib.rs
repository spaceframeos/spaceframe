use bitvec::prelude::*;

pub mod constants;
pub mod core;
pub mod f1_calculator;
pub mod fx_calculator;
pub mod utils;
pub mod proofs;
pub mod collation;
pub mod bits;
pub mod storage;

pub type Bits = BitVec<Lsb0, u8>;
pub type BitsSlice = BitSlice<Lsb0, u8>;
