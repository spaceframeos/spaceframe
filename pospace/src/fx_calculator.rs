use bitvec::prelude::*;

use crate::{constants::PARAM_EXT, Bits, BitsSlice};

#[derive(Debug)]
pub struct FXCalculator {
    k: usize,
    f_size: usize,
}

impl FXCalculator {
    pub fn new(k: usize) -> Self {
        FXCalculator {
            k,
            f_size: k + PARAM_EXT,
        }
    }

    pub fn calculate_fn(&self, input: &[&BitsSlice], y: &BitsSlice) -> Bits {
        let half_len = input.len() / 2;
        let mut first_half: Bits = BitVec::new();
        let mut last_half: Bits = BitVec::new();
        for slice in &input[..half_len] {
            first_half.extend_from_bitslice(slice);
        }
        for slice in &input[half_len..] {
            last_half.extend_from_bitslice(slice);
        }
        calculate_blake_hash(y, &first_half, &last_half)[..self.f_size].to_bitvec()
    }

    // pub fn collate_n(&self, input: &[&BitsSlice]) -> Result<Bits, ()> {
    //     match input.len() {
    //         1 => {
    //             return Ok(input[0].to_bitvec());
    //         }
    //         2 => {
    //             let mut out = input[0].to_bitvec();
    //             out.extend_from_bitslice(input[1]);
    //             return Ok(out);
    //         }
    //         4 => {
    //             let mut out = input[0].to_bitvec();
    //             out.extend_from_bitslice(input[1]);
    //             out.extend_from_bitslice(input[2]);
    //             out.extend_from_bitslice(input[3]);
    //             return Ok(out);
    //         }
    //         8 => {
    //             return Ok(calculate_blake_hash(
    //                 &self.collate_n(&input[0..4]).unwrap(),
    //                 &self.collate_n(&input[4..8]).unwrap(),
    //                 &self.calculate_fn(&input[0..4]),
    //             )[self.f_size..self.f_size + 4 * self.k]
    //                 .to_bitvec());
    //         }
    //         16 => {
    //             return Ok(calculate_blake_hash(
    //                 &self.collate_n(&input[0..8]).unwrap(),
    //                 &self.collate_n(&input[8..16]).unwrap(),
    //                 &self.calculate_fn(&input[0..8]),
    //             )[self.f_size..self.f_size + 3 * self.k]
    //                 .to_bitvec());
    //         }
    //         32 => {
    //             return Ok(calculate_blake_hash(
    //                 &self.collate_n(&input[0..16]).unwrap(),
    //                 &self.collate_n(&input[16..32]).unwrap(),
    //                 &self.calculate_fn(&input[0..16]),
    //             )[self.f_size..self.f_size + 2 * self.k]
    //                 .to_bitvec());
    //         }
    //         _ => {
    //             return Err(());
    //         }
    //     }
    // }
}

pub fn calculate_blake_hash(y: &BitsSlice, l: &BitsSlice, r: &BitsSlice) -> Bits {
    let mut hasher = blake3::Hasher::new();
    let mut input: Bits = BitVec::new();
    input.extend_from_bitslice(y);
    input.extend_from_bitslice(l);
    input.extend_from_bitslice(r);
    hasher.update(input.as_raw_slice());
    let hash = hasher.finalize();
    hash.as_bytes().view_bits().to_bitvec()
}

// #[cfg(test)]
// mod tests {
//     use crate::utils::{from_bits, to_bits};

//     use super::*;
//     use rstest::rstest;

//     #[ignore]
//     #[rstest]
//     #[case(2, 16, 0xa, 0x204f, 0x20a61a, 0x2af546, 0x44cb204f)]
//     fn verify_functions(
//         #[case] t: u8,
//         #[case] k: u64,
//         #[case] L: u64,
//         #[case] R: u64,
//         #[case] y1: u64,
//         #[case] y: u64,
//         #[case] c: u64,
//     ) {
//         let y1 = to_bits(y1, k as usize + 6);
//         let L = to_bits(L, k as usize);
//         let R = to_bits(R, k as usize);
//         let res = &calculate_blake_hash(&y1, &L, &R)[..k as usize + PARAM_EXT];
//         println!("{}", &res[..k as usize + 6]);
//         println!("{}", y);
//         assert_eq!(from_bits(&res), y);
//     }
// }
