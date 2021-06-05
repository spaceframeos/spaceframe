use bitvec::prelude::*;

use crate::{Bits, BitsSlice};

#[derive(Debug)]
pub struct BitsWrapper {
    pub bits: Bits,
    pub value: u64,
}

impl BitsWrapper {
    pub fn new(bits: Bits) -> Self {
        BitsWrapper {
            value: from_bits(&bits),
            bits,
        }
    }
}

pub fn to_bits(input: u64, size: usize) -> Bits {
    let mut input_bits = input.to_le_bytes().view_bits::<Lsb0>()[..size].to_bitvec();
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