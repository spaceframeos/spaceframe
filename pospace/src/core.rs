use rayon::prelude::*;
use std::sync::mpsc::channel;

use crate::{
    bits::{to_bits, BitsWrapper},
    constants::{PARAM_B, PARAM_BC, PARAM_C, PARAM_EXT, PARAM_M},
    f1_calculator::F1Calculator,
    fx_calculator::FXCalculator,
};

pub struct PlotEntry {
    pub bits_wrapper: BitsWrapper,
    pub x1: u64,
    pub x2: u64,
}

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

    /// ```cpp
    /// bool CheckMatch(int64_t yl, int64_t yr)
    /// {
    ///     int64_t bl = yl / kBC;
    ///     int64_t br = yr / kBC;
    ///     if (bl + 1 != br)
    ///         return false;  // Buckets don't match
    ///     for (int64_t m = 0; m < kExtraBitsPow; m++) {
    ///         if ((((yr % kBC) / kC - ((yl % kBC) / kC)) - m) % kB == 0) {
    ///             int64_t c_diff = 2 * m + bl % 2;
    ///             c_diff *= c_diff;
    ///
    ///             if ((((yr % kBC) % kC - ((yl % kBC) % kC)) - c_diff) % kC == 0) {
    ///                 return true;
    ///             }
    ///         }
    ///     }
    ///     return false;
    /// }
    /// ```
    pub fn matching(&self, l: &BitsWrapper, r: &BitsWrapper) -> bool {
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

        //   For any 0 <= m < kExtraBitsPow:
        //   yl / kBC + 1 = yR / kBC   AND
        //   (yr % kBC) / kC - (yl % kBC) / kC = m   (mod kB)  AND
        //   (yr % kBC) % kC - (yl % kBC) % kC = (2m + (yl/kBC) % 2)^2   (mod kC)
        //

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
                let mut c_diff = 2 * m + bl % 2;
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
                        if self.matching(fx1, fx2) {
                            let f2x = self.fx_calculator.calculate_fn(
                                &[&to_bits(i, self.k), &to_bits(i, self.k)],
                                &fx1.bits,
                            );
                            s.send((BitsWrapper::new(f2x), i, j)).unwrap();
                        }
                    }
                }
            });

        let table2: Vec<(BitsWrapper, u64, u64)> = receiver.iter().collect();

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

                        if self.matching(fx1, fx2) {
                            let f2x = self.fx_calculator.calculate_fn(
                                &[
                                    &to_bits(entry1.1, self.k),
                                    &to_bits(entry1.2, self.k),
                                    &to_bits(entry2.1, self.k),
                                    &to_bits(entry2.2, self.k),
                                ],
                                &fx1.bits,
                            );
                            s.send((
                                BitsWrapper::new(f2x),
                                entry1.1,
                                entry1.2,
                                entry2.1,
                                entry2.2,
                            ))
                            .unwrap();
                        }
                    }
                }
            });

        let table3: Vec<(BitsWrapper, u64, u64, u64, u64)> = receiver.iter().collect();

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

                        if self.matching(fx1, fx2) {
                            let f2x = self.fx_calculator.calculate_fn(
                                &[
                                    &to_bits(entry1.1, self.k),
                                    &to_bits(entry1.2, self.k),
                                    &to_bits(entry1.3, self.k),
                                    &to_bits(entry1.4, self.k),
                                    &to_bits(entry2.1, self.k),
                                    &to_bits(entry2.2, self.k),
                                    &to_bits(entry2.3, self.k),
                                    &to_bits(entry2.4, self.k),
                                ],
                                &fx1.bits,
                            );
                            s.send((
                                BitsWrapper::new(f2x),
                                entry1.1,
                                entry1.2,
                                entry1.3,
                                entry1.4,
                                entry2.1,
                                entry1.2,
                                entry1.3,
                                entry1.4,
                            ))
                            .unwrap();
                        }
                    }
                }
            });

        let table4: Vec<(BitsWrapper, u64, u64, u64, u64, u64, u64, u64, u64)> = receiver.iter().collect();

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
