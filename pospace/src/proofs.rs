use bitvec::prelude::*;

use crate::{BitsSlice, f1_calculator::calculate_f1, fx_calculator::calculate_f2, utils::matching};

pub fn verify_prove(x1: u64, x2: u64, challenge: &BitsSlice, k: usize) -> bool {
    let x1_bytes = x1.to_be_bytes();
    let x1_bits = &x1_bytes.view_bits()[64 - k as usize..];
    let x2_bytes = x2.to_be_bytes();
    let x2_bits = &x2_bytes.view_bits()[64 - k as usize..];
    let f1x1 = calculate_f1(x1_bits, k);
    let f1x2 = calculate_f1(x2_bits, k);
    if matching(&f1x1, &f1x2) {
        let f2x1 = &calculate_f2(&x1_bits, &x2_bits, &f1x1)[..k as usize];
        return f2x1 == challenge;
    }
    return false;
}