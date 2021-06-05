use std::time::{Duration, Instant};

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
        let mut table1 = Vec::new();
        let mut table2 = Vec::new();
        let mut table3 = Vec::new();
        let mut table4 = Vec::new();

        for x in 0..(2u64).pow(self.k as u32) {
            let fx = self.f1_calculator.calculate_f1(&to_bits(x, self.k), x);
            table1.push(BitsWrapper::new(fx));
        }
        println!("Table 1 len: {}", table1.len());

        let mut counter = 0;

        let mut timer = Instant::now();
        let total = (2u64).pow(self.k as u32);

        // Table 2
        'outer1: for x1 in 0..(2u64).pow(self.k as u32) {
            for x2 in 0..(2u64).pow(self.k as u32) {
                if x1 != x2 {
                    if timer.elapsed() >= Duration::from_secs(1) {
                        let percent = ((x1 * total + x2) as f64 / total.pow(2) as f64) * 100f64;
                        println!(
                            "{:.2}% complete, table fill: {:.2}%",
                            percent,
                            table2.len() as f64 / total as f64 * 100.0
                        );
                        timer = Instant::now();
                    }
                    let fx1 = &table1[x1 as usize];
                    let fx2 = &table1[x2 as usize];
                    if self.matching(fx1, fx2) {
                        let f2x = self
                            .fx_calculator
                            .calculate_fn(&[&to_bits(x1, self.k), &to_bits(x1, self.k)], &fx1.bits);
                        counter += 1;
                        table2.push((BitsWrapper::new(f2x), x1, x2));

                        if counter == (2u64).pow(self.k as u32) {
                            break 'outer1;
                        }
                    }
                }
            }
        }

        println!(
            "Table 2 len: {} ({:.2}%)",
            table2.len(),
            table2.len() as f64 / total as f64 * 100.0
        );
        counter = 0;

        timer = Instant::now();

        // Table 3
        'outer2: for i in 0..table2.len() {
            for j in 0..table2.len() {
                if i != j {
                    if timer.elapsed() >= Duration::from_secs(2) {
                        let percent =
                            ((i as u64 * total + j as u64) as f64 / total.pow(2) as f64) * 100f64;
                        println!(
                            "{:.2}% complete, table fill: {:.2}%",
                            percent,
                            table3.len() as f64 / total as f64 * 100.0
                        );
                        timer = Instant::now();
                    }
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
                        counter += 1;
                        table3.push((
                            BitsWrapper::new(f2x),
                            entry1.1,
                            entry1.2,
                            entry2.1,
                            entry2.2,
                        ));

                        if counter == (2u64).pow(self.k as u32) {
                            break 'outer2;
                        }
                    }
                }
            }
        }

        println!(
            "Table 3 len: {} ({:.2}%)",
            table3.len(),
            table3.len() as f64 / total as f64 * 100.0
        );
        counter = 0;

        timer = Instant::now();

        // Table 4
        'outer3: for i in 0..table3.len() {
            for j in 0..table3.len() {
                if i != j {
                    if timer.elapsed() >= Duration::from_secs(2) {
                        let percent =
                            ((i as u64 * total + j as u64) as f64 / total.pow(2) as f64) * 100f64;
                        println!(
                            "{:.2}% complete, table fill: {:.2}%",
                            percent,
                            table4.len() as f64 / total as f64 * 100.0
                        );
                        timer = Instant::now();
                    }

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
                        counter += 1;
                        table4.push((
                            BitsWrapper::new(f2x),
                            entry1.1,
                            entry1.2,
                            entry1.3,
                            entry1.4,
                            entry2.1,
                            entry1.2,
                            entry1.3,
                            entry1.4,
                        ));

                        if counter == (2u64).pow(self.k as u32) {
                            break 'outer3;
                        }
                    }
                }
            }
        }
        println!(
            "Table 2 len: {} ({:.2}%)",
            table2.len(),
            table2.len() as f64 / total as f64 * 100.0
        );
        println!(
            "Table 3 len: {} ({:.2}%)",
            table3.len(),
            table3.len() as f64 / total as f64 * 100.0
        );
        println!(
            "Table 4 len: {} ({:.2}%)",
            table4.len(),
            table4.len() as f64 / total as f64 * 100.0
        );
    }
}
