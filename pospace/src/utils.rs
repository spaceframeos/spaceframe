use crate::constants::{PARAM_BC, PARAM_C};

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
