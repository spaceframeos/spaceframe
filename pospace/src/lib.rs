use bitvec::prelude::*;

pub mod constants;
pub mod core;
pub mod f1_calculator;
pub mod fx_calculator;
pub mod utils;
pub mod proofs;

pub type Bits = BitVec<Msb0, u8>;
pub type BitsSlice = BitSlice<Msb0, u8>;

// pub fn init_pos(k: usize) {
//     let chall = b"caaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
//     let target = &chall.view_bits::<Msb0>()[..k as usize];
//     let mut proves_count = 0;
//     let mut proves = vec![];
//     for x in table2 {
//         if x.0[..k as usize] == target {
//             proves_count += 1;
//             let el = &x;
//             let x1 = el.1;
//             let x2 = el.2;
//             proves.push((x1, x2));
//             println!("Prove {}: x1 = {}, x2 = {}", proves_count, x1, x2);
//         }
//     }
//     println!("Target: {}", target);
//     println!("Proves count: {}", proves_count);

//     for (index, prove) in proves.into_iter().enumerate() {
//         println!(
//             "Prove {}: {}",
//             index,
//             verify_prove(prove.0, prove.1, target, k)
//         );
//     }
// }
