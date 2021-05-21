use bitvec::prelude::*;

use crate::{Bits, BitsSlice};

pub fn fx_blake_hash(y: &BitsSlice, l: &BitsSlice, r: &BitsSlice) -> Bits {
    let mut hasher = blake3::Hasher::new();
    hasher.update(y.as_raw_slice());
    hasher.update(l.as_raw_slice());
    hasher.update(r.as_raw_slice());
    let hash = hasher.finalize();
    hash.as_bytes().view_bits().to_bitvec()
}

pub fn calculate_f2(x1: &BitsSlice, x2: &BitsSlice, f1x: &BitsSlice) -> Bits {
    fx_blake_hash(x1, x2, f1x)[..16].to_bitvec()
}
