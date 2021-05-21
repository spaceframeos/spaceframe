use bitvec::prelude::*;

use crate::{BitsSlice, constants::{PARAM_BC, PARAM_C, PARAM_M}};

pub fn bucket_id(x: &BitsSlice) -> u64 {
    (x.load_be::<u64>() as f64 / PARAM_BC as f64).floor() as u64
}

pub fn divmod(x: u64, m: u64) -> (u64, u64) {
    (x.div_euclid(m), x.rem_euclid(m))
}

pub fn b_id(x: &BitsSlice) -> u64 {
    divmod(x.load_be::<u64>() % PARAM_BC, PARAM_C).0
}

pub fn c_id(x: &BitsSlice) -> u64 {
    divmod(x.load_be::<u64>() % PARAM_BC, PARAM_C).1
}

pub fn matching(l: &BitsSlice, r: &BitsSlice) -> bool {
    let bucket_id_l = bucket_id(l);
    if bucket_id_l + 1 != bucket_id(r) {
        return false;
    }

    let bidr = b_id(r) as i64;
    let bidl = b_id(l) as i64;
    let cidr = c_id(r) as i64;
    let cidl = c_id(l) as i64;

    let a = (bidr - bidl).rem_euclid(PARAM_M as i64);
    let b = (cidr - cidl).rem_euclid(PARAM_M as i64);

    for m in 0..PARAM_M {
        if a == m as i64 {
            if b == ((2 * m + (bucket_id_l % 2)).pow(2) % PARAM_C) as i64 {
                return true;
            }
        }
    }

    return false;
}