use bitvec::prelude::*;

use crate::{f1_calculator::calculate_f1, fx_calculator::calculate_f2, proofs::verify_prove, utils::matching};

pub mod constants;
pub mod core;
pub mod f1_calculator;
pub mod fx_calculator;
pub mod utils;
pub mod proofs;

pub type Bits = BitVec<Msb0, u8>;
pub type BitsSlice = BitSlice<Msb0, u8>;

pub fn init_pos(k: usize) {
    let mut table1 = vec![];
    let mut table2 = vec![];
    for x in 0..(2u64).pow(k as u32) {
        let fx = calculate_f1(&x.to_be_bytes().view_bits()[(64 - k) as usize..], k);
        table1.push(fx);
    }
    let mut counter = 0;

    // Table 2
    'outer: for x1 in 0..(2 as u64).pow(k as u32) {
        for x2 in x1..(2 as u64).pow(k as u32) {
            if x1 != x2 {
                let fx1 = &table1[x1 as usize];
                let fx2 = &table1[x2 as usize];
                if matching(fx1, fx2) {
                    let f2x = calculate_f2(
                        &x1.to_be_bytes().view_bits()[(64 - k) as usize..],
                        &x2.to_be_bytes().view_bits()[(64 - k) as usize..],
                        fx1,
                    );
                    // println!("f2x = {}, pos = {}, offset = {}", f2x, pos, offset);
                    counter += 1;
                    table2.push((f2x, x1, x2));

                    if counter == (2 as u32).pow(k as u32) {
                        break 'outer;
                    }
                }
            }
        }
    }
    println!("Count: {}", counter);
    let chall = b"caaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let target = &chall.view_bits::<Msb0>()[..k as usize];
    let mut proves_count = 0;
    let mut proves = vec![];
    for x in table2 {
        if x.0[..k as usize] == target {
            proves_count += 1;
            let el = &x;
            let x1 = el.1;
            let x2 = el.2;
            proves.push((x1, x2));
            println!("Prove {}: x1 = {}, x2 = {}", proves_count, x1, x2);
        }
    }
    println!("Target: {}", target);
    println!("Proves count: {}", proves_count);

    for (index, prove) in proves.into_iter().enumerate() {
        println!(
            "Prove {}: {}",
            index,
            verify_prove(prove.0, prove.1, target, k)
        );
    }
}
