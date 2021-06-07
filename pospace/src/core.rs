use rayon::prelude::*;
use std::sync::mpsc::channel;

use crate::{
    bits::{to_bits, BitsWrapper},
    constants::{PARAM_B, PARAM_BC, PARAM_C, PARAM_EXT, PARAM_M},
    f1_calculator::F1Calculator,
    fx_calculator::FXCalculator,
};

#[derive(Debug)]
pub struct PoSpace {
    plot_seed: Vec<u8>,
    k: usize,
    f1_calculator: F1Calculator,
    fx_calculator: FXCalculator,
}

impl PoSpace {
    pub fn new(k: usize, plot_seed: &[u8]) -> Self {
        PoSpace {
            plot_seed: plot_seed.to_vec(),
            k,
            f1_calculator: F1Calculator::new(k, &plot_seed),
            fx_calculator: FXCalculator::new(k),
        }
    }

    pub fn matching_naive(&self, l: &BitsWrapper, r: &BitsWrapper) -> bool {
        assert_eq!(
            self.k + PARAM_EXT,
            l.bits.len(),
            "l must be {} bits",
            self.k + PARAM_EXT
        );
        assert_eq!(
            self.k + PARAM_EXT,
            r.bits.len(),
            "r must be {} bits",
            self.k + PARAM_EXT
        );

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

    pub fn run_phase_1(&self) {
        let table_size = 2u64.pow(self.k as u32);

        let (sender, receiver) = channel();

        (0..table_size)
            .into_par_iter()
            .for_each_with(sender, |s, x| {
                let fx = self.f1_calculator.calculate_f1(&to_bits(x, self.k), x);
                s.send(BitsWrapper::new(fx)).unwrap();
            });

        let table1: Vec<BitsWrapper> = receiver.iter().collect();
        println!("Table 1 len: {}", table1.len());

        // Table 2
        let (sender, receiver) = channel();

        (0..table_size)
            .into_par_iter()
            .for_each_with(sender, |s, i| {
                for j in 0..table_size {
                    if i != j {
                        let fx1 = &table1[i as usize];
                        let fx2 = &table1[j as usize];
                        if self.matching_naive(fx1, fx2) {
                            let f2x = self.fx_calculator.calculate_fn(
                                &[&to_bits(i, self.k), &to_bits(i, self.k)],
                                &fx1.bits,
                            );
                            s.send((
                                BitsWrapper::new(f2x),
                                BitsWrapper::from(i, self.k),
                                BitsWrapper::from(j, self.k),
                            ))
                            .unwrap();
                        }
                    }
                }
            });

        let table2: Vec<(BitsWrapper, BitsWrapper, BitsWrapper)> = receiver.iter().collect();

        println!(
            "Table 2 len: {} ({:.2}%)",
            table2.len(),
            table2.len() as f64 / table_size as f64 * 100.0
        );

        // Table 3
        let (sender, receiver) = channel();

        (0..table2.len())
            .into_par_iter()
            .for_each_with(sender, |s, i| {
                for j in 0..table2.len() {
                    if i != j {
                        let entry1 = &table2[i];
                        let entry2 = &table2[j];
                        let fx1 = &entry1.0;
                        let fx2 = &entry2.0;

                        if self.matching_naive(fx1, fx2) {
                            let f2x = self.fx_calculator.calculate_fn(
                                &[
                                    &entry1.1.bits,
                                    &entry1.2.bits,
                                    &entry2.1.bits,
                                    &entry2.2.bits,
                                ],
                                &fx1.bits,
                            );
                            s.send((
                                BitsWrapper::new(f2x),
                                entry1.1.clone(),
                                entry1.2.clone(),
                                entry2.1.clone(),
                                entry2.2.clone(),
                            ))
                            .unwrap();
                        }
                    }
                }
            });

        let table3: Vec<(
            BitsWrapper,
            BitsWrapper,
            BitsWrapper,
            BitsWrapper,
            BitsWrapper,
        )> = receiver.iter().collect();

        println!(
            "Table 3 len: {} ({:.2}%)",
            table3.len(),
            table3.len() as f64 / table_size as f64 * 100.0
        );

        // Table 4
        let (sender, receiver) = channel();

        (0..table3.len())
            .into_par_iter()
            .for_each_with(sender, |s, i| {
                for j in 0..table3.len() {
                    if i != j {
                        let entry1 = &table3[i];
                        let entry2 = &table3[j];
                        let fx1 = &entry1.0;
                        let fx2 = &entry2.0;

                        if self.matching_naive(fx1, fx2) {
                            let f2x = self.fx_calculator.calculate_fn(
                                &[
                                    &entry1.1.bits,
                                    &entry1.2.bits,
                                    &entry1.3.bits,
                                    &entry1.4.bits,
                                    &entry2.1.bits,
                                    &entry2.2.bits,
                                    &entry2.3.bits,
                                    &entry2.4.bits,
                                ],
                                &fx1.bits,
                            );
                            s.send((
                                BitsWrapper::new(f2x),
                                entry1.1.clone(),
                                entry1.2.clone(),
                                entry1.3.clone(),
                                entry1.4.clone(),
                                entry2.1.clone(),
                                entry1.2.clone(),
                                entry1.3.clone(),
                                entry1.4.clone(),
                            ))
                            .unwrap();
                        }
                    }
                }
            });

        let table4: Vec<(
            BitsWrapper,
            BitsWrapper,
            BitsWrapper,
            BitsWrapper,
            BitsWrapper,
            BitsWrapper,
            BitsWrapper,
            BitsWrapper,
            BitsWrapper,
        )> = receiver.iter().collect();

        println!(
            "Table 4 len: {} ({:.2}%)",
            table4.len(),
            table4.len() as f64 / table_size as f64 * 100.0
        );

        println!("\nFinal tables:");
        println!(
            "Table 2 len: {} ({:.2}%)",
            table2.len(),
            table2.len() as f64 / table_size as f64 * 100.0
        );
        println!(
            "Table 3 len: {} ({:.2}%)",
            table3.len(),
            table3.len() as f64 / table_size as f64 * 100.0
        );
        println!(
            "Table 4 len: {} ({:.2}%)",
            table4.len(),
            table4.len() as f64 / table_size as f64 * 100.0
        );
    }
}
