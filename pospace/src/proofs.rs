use bitvec::prelude::*;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{Bits, BitsSlice, bits::BitsWrapper, constants::{PARAM_B, PARAM_BC, PARAM_C, PARAM_EXT, PARAM_M}, f1_calculator::F1Calculator, fx_calculator::FXCalculator};

pub struct Proof {
    items: Vec<BitsWrapper>,
    challenge: Bits,
    k: usize,
}

pub fn matching_naive(l: &BitsWrapper, r: &BitsWrapper) -> bool {
    let k_bc = PARAM_BC as i64;
    let k_b = PARAM_B as i64;
    let k_c = PARAM_C as i64;

    let yl = l.value as i64;
    let yr = r.value as i64;

    let bl = yl / k_bc;
    let br = yr / k_bc;

    if bl + 1 != br {
        return false;
    }

    for m in 0..PARAM_M {
        let m = m as i64;
        if (((yr % k_bc) / k_c - ((yl % k_bc) / k_c)) - m) % k_b == 0 {
            let mut c_diff = 2 * m + (bl % 2);
            c_diff *= c_diff;

            if (((yr % k_bc) % k_c - ((yl % k_bc) % k_c)) - c_diff) % k_c == 0 {
                return true;
            }
        }
    }
    return false;
}

pub fn verify_prove(proof: Proof, plot_seed: &[u8]) -> bool {
    let f1calc = F1Calculator::new(proof.k, plot_seed);
    let f1y: Vec<BitsWrapper> = proof
        .items
        .par_iter()
        .map(|b| BitsWrapper::new(f1calc.calculate_f1(&b)))
        .collect();
    println!("Table 1 check:");
    for i in (0..f1y.len()).step_by(2) {
        let is_match = matching_naive(&f1y[i], &f1y[i + 1]);
        println!("Matching: {}", is_match);
        if !is_match {
            return false;
        }
    }
    println!("Table 2 check:");
    let fxcalc = FXCalculator::new(proof.k);
    let mut table2 = Vec::new();
    for i in (0..f1y.len()).step_by(2) {
        let f2y = fxcalc.calculate_fn(&[
            &proof.items[i].bits,
            &proof.items[i + 1].bits
        ], &f1y[i].bits);
        table2.push(BitsWrapper::new(f2y));
    }

    for i in (0..table2.len()).step_by(2) {
        let is_match = matching_naive(&table2[i], &table2[i + 1]);
        println!("Matching: {}", is_match);
        if !is_match {
            return false;
        }
    }

    println!("{:?}", table2.len());
    
    let mut table3 = Vec::new();
    for i in (0..f1y.len()).step_by(4) {
        let f3y = fxcalc.calculate_fn(&[
            &proof.items[i].bits,
            &proof.items[i + 1].bits,
            &proof.items[i + 2].bits,
            &proof.items[i + 3].bits,
        ], &table2[i / 2].bits);
        table3.push(BitsWrapper::new(f3y));
    }

    for i in (0..table3.len()).step_by(2) {
        let is_match = matching_naive(&table3[i], &table3[i + 1]);
        println!("Matching: {}", is_match);
        if !is_match {
            return false;
        }
    }

    return true;
}

#[cfg(test)]
mod tests {
    use crate::{bits::to_bits, core::PoSpace};

    use super::*;

    #[test]
    fn test_proving() {
        let k = 12;
        let plot_seed = b"abcdabcdabcdabcdabcdabcdabcdabcd";
        let mut pos = PoSpace::new(k, plot_seed);
        let challenge = b"this is the challenge".view_bits::<Lsb0>()[..k].to_bitvec();
        pos.run_phase_1();
        let item = pos.table4.iter().find(|x| {
            return x.0.bits[..k] == challenge;
        });

        if item.is_some() {
            println!("Proof found");
            let item = item.unwrap();
            println!("Verifying proof...");
            let verified = verify_prove(Proof {
                items: vec![
                    item.1.clone(),
                    item.2.clone(),
                    item.3.clone(),
                    item.4.clone(),
                    item.5.clone(),
                    item.6.clone(),
                    item.7.clone(),
                    item.8.clone(),
                ],
                challenge: to_bits(42, 14),
                k,
            }, plot_seed);
            assert!(verified);
        } else {
            assert!(false, "No proof found :(");
        }
    }
}
