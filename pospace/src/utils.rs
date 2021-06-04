use bitvec::prelude::*;

use crate::{Bits, BitsSlice, constants::{PARAM_BC, PARAM_C}};

pub fn bucket_id(x: u64) -> u64 {
    x / PARAM_BC
}

pub fn divmod(x: u64, m: u64) -> (u64, u64) {
    (x.div_euclid(m), x.rem_euclid(m))
}

pub fn b_id(x: u64) -> u64 {
    divmod(x % PARAM_BC, PARAM_C).0
}

pub fn c_id(x: u64) -> u64 {
    divmod(x % PARAM_BC, PARAM_C).1
}

pub fn to_bits(input: u64, size: usize) -> Bits {
    let input_bytes = input.to_le_bytes();
    let mut input_bits = input_bytes.view_bits::<Lsb0>()[..size].to_bitvec();
    input_bits.reverse();
    input_bits
}

pub fn from_bits(input: &BitsSlice) -> u64 {
    let mut vec = input.to_bitvec();
    vec.reverse();
    vec.load_le::<u64>()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_to_bits() {
        assert_eq!(to_bits(5, 10), bitvec![0, 0, 0, 0, 0, 0, 0, 1, 0, 1]);
        assert_eq!(to_bits(13, 10), bitvec![0, 0, 0, 0, 0, 0, 1, 1, 0, 1]);
        assert_eq!(to_bits(0xab, 10), bitvec![0, 0, 1, 0, 1, 0, 1, 0, 1, 1]);
        assert_eq!(to_bits(0xabcd, 16), bitvec![1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 0, 1]);
    }

    #[test]
    fn test_from_bits() {
        assert_eq!(from_bits(&bitvec![Lsb0, u8; 0, 0, 0, 0, 0, 0, 0, 1, 0, 1]), 5);
        assert_eq!(from_bits(&bitvec![Lsb0, u8; 0, 0, 0, 0, 0, 0, 1, 1, 0, 1]), 13);
        assert_eq!(from_bits(&bitvec![Lsb0, u8; 0, 0, 1, 0, 1, 0, 1, 0, 1, 1]), 0xab);
    }

    #[test]
    fn test_bits_slice() {
        assert_eq!(to_bits(0xabcd, 16)[4..13], bitvec![Lsb0, u8; 1, 0, 1, 1, 1, 1, 0, 0, 1]);
        assert_eq!(to_bits(0x7b91, 16)[9..], bitvec![Lsb0, u8; 0, 0, 1, 0, 0, 0, 1]);
    }
}