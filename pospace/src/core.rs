use crossbeam::channel::bounded;
use log::{info, warn};
use rayon::prelude::*;
use std::{
    fs::{create_dir_all, remove_dir_all},
    path::Path,
};

use crate::{
    bits::BitsWrapper,
    constants::{PARAM_B, PARAM_BC, PARAM_C, PARAM_EXT, PARAM_M},
    f1_calculator::F1Calculator,
    fx_calculator::FXCalculator,
    storage::{sort_table_on_disk, store_table1_part, Table1Entry, ENTRIES_PER_CHUNK},
};

#[derive(Debug)]
pub struct PoSpace {
    plot_seed: Vec<u8>,
    k: usize,
    f1_calculator: F1Calculator,
    fx_calculator: FXCalculator,

    pub table1: Vec<(BitsWrapper, BitsWrapper)>,
    pub table2: Vec<(BitsWrapper, BitsWrapper, BitsWrapper)>,
    pub table3: Vec<(
        BitsWrapper,
        BitsWrapper,
        BitsWrapper,
        BitsWrapper,
        BitsWrapper,
    )>,
    pub table4: Vec<(
        BitsWrapper,
        BitsWrapper,
        BitsWrapper,
        BitsWrapper,
        BitsWrapper,
        BitsWrapper,
        BitsWrapper,
        BitsWrapper,
        BitsWrapper,
    )>,
}

impl PoSpace {
    pub fn new(k: usize, plot_seed: &[u8]) -> Self {
        PoSpace {
            plot_seed: plot_seed.to_vec(),
            k,
            f1_calculator: F1Calculator::new(k, &plot_seed),
            fx_calculator: FXCalculator::new(k),
            table1: Vec::new(),
            table2: Vec::new(),
            table3: Vec::new(),
            table4: Vec::new(),
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

    pub fn run_phase_1(&mut self) {
        // Clear data folder
        match remove_dir_all("data") {
            Ok(_) => {
                info!("Cleaning data folder");
            }
            Err(e) => {
                warn!("Cannot clean data folder: {}", e);
            }
        }
        create_dir_all("data").ok();
        info!("Data dir cleaned");

        let table_size = 2u64.pow(self.k as u32);

        info!("Calculating table 1 ...");

        rayon::scope(|s| {
            let (sender, receiver) = bounded(ENTRIES_PER_CHUNK);

            s.spawn(|_| {
                (0..table_size)
                    .into_par_iter()
                    .for_each_with(sender, |s, x| {
                        let x_wrapped = BitsWrapper::from(x, self.k);
                        let fx = self.f1_calculator.calculate_f1(&x_wrapped);
                        s.send((BitsWrapper::new(fx), x_wrapped)).unwrap();
                    });
                info!("Calculating finished");
            });

            let mut buffer = Vec::new();
            let mut counter = 1;

            while let Ok(data) = receiver.recv() {
                buffer.push(Table1Entry {
                    x: data.1.value,
                    y: data.0.value,
                });

                if buffer.len() % (1024 * 1024) == 0 {
                    info!(
                        "Progess: {:.3}%",
                        (buffer.len() + (counter - 1) * ENTRIES_PER_CHUNK) as f64
                            / (table_size as usize) as f64
                            * 100 as f64
                    );
                }

                if buffer.len() == ENTRIES_PER_CHUNK {
                    info!("Wrinting raw data to disk ...");
                    store_table1_part(&buffer, Path::new("data"), counter);
                    counter += 1;
                    buffer.clear();
                }
            }

            if buffer.len() > 0 {
                store_table1_part(&buffer, Path::new("data"), counter);
            }
        });

        info!("Table 1 raw data written");
        info!("Starting to sort table 1 on disk ...");

        sort_table_on_disk::<Table1Entry>(1, Path::new("data"), ENTRIES_PER_CHUNK);

        info!("Table 1 sorted on disk");

        // Table 2
        // let (sender, receiver) = bounded(ENTRIES_PER_CHUNK);

        // (0..self.table1.len())
        //     .into_par_iter()
        //     .for_each_with(sender, |s, i| {
        //         for j in 0..self.table1.len() {
        //             if i != j {
        //                 let entry1 = &self.table1[i];
        //                 let entry2 = &self.table1[j];
        //                 let fx1 = &entry1.0;
        //                 let fx2 = &entry2.0;
        //                 if self.matching_naive(fx1, fx2) {
        //                     let f2x = self
        //                         .fx_calculator
        //                         .calculate_fn(&[&entry1.1.bits, &entry2.1.bits], &fx1.bits);
        //                     s.send((BitsWrapper::new(f2x), entry1.1.clone(), entry2.1.clone()))
        //                         .unwrap();
        //                 }
        //             }
        //         }
        //     });

        // self.table2 = receiver.iter().collect();
        // self.table2.sort_by(|a, b| a.0.value.cmp(&b.0.value));

        // println!(
        //     "Table 2 len: {} ({:.2}%)",
        //     self.table2.len(),
        //     self.table2.len() as f64 / table_size as f64 * 100.0
        // );

        // // Table 3
        // let (sender, receiver) = bounded(ENTRIES_PER_CHUNK);

        // (0..self.table2.len())
        //     .into_par_iter()
        //     .for_each_with(sender, |s, i| {
        //         for j in 0..self.table2.len() {
        //             if i != j {
        //                 let entry1 = &self.table2[i];
        //                 let entry2 = &self.table2[j];
        //                 let fx1 = &entry1.0;
        //                 let fx2 = &entry2.0;

        //                 if self.matching_naive(fx1, fx2) {
        //                     let f2x = self.fx_calculator.calculate_fn(
        //                         &[
        //                             &entry1.1.bits,
        //                             &entry1.2.bits,
        //                             &entry2.1.bits,
        //                             &entry2.2.bits,
        //                         ],
        //                         &fx1.bits,
        //                     );
        //                     s.send((
        //                         BitsWrapper::new(f2x),
        //                         entry1.1.clone(),
        //                         entry1.2.clone(),
        //                         entry2.1.clone(),
        //                         entry2.2.clone(),
        //                     ))
        //                     .unwrap();
        //                 }
        //             }
        //         }
        //     });

        // self.table3 = receiver.iter().collect();
        // self.table3.sort_by(|a, b| a.0.value.cmp(&b.0.value));

        // println!(
        //     "Table 3 len: {} ({:.2}%)",
        //     self.table3.len(),
        //     self.table3.len() as f64 / table_size as f64 * 100.0
        // );

        // // Table 4
        // let (sender, receiver) = bounded(ENTRIES_PER_CHUNK);

        // (0..self.table3.len())
        //     .into_par_iter()
        //     .for_each_with(sender, |s, i| {
        //         for j in 0..self.table3.len() {
        //             if i != j {
        //                 let entry1 = &self.table3[i];
        //                 let entry2 = &self.table3[j];
        //                 let fx1 = &entry1.0;
        //                 let fx2 = &entry2.0;

        //                 if self.matching_naive(fx1, fx2) {
        //                     let f2x = self.fx_calculator.calculate_fn(
        //                         &[
        //                             &entry1.1.bits,
        //                             &entry1.2.bits,
        //                             &entry1.3.bits,
        //                             &entry1.4.bits,
        //                             &entry2.1.bits,
        //                             &entry2.2.bits,
        //                             &entry2.3.bits,
        //                             &entry2.4.bits,
        //                         ],
        //                         &fx1.bits,
        //                     );
        //                     s.send((
        //                         BitsWrapper::new(f2x),
        //                         entry1.1.clone(),
        //                         entry1.2.clone(),
        //                         entry1.3.clone(),
        //                         entry1.4.clone(),
        //                         entry2.1.clone(),
        //                         entry2.2.clone(),
        //                         entry2.3.clone(),
        //                         entry2.4.clone(),
        //                     ))
        //                     .unwrap();
        //                 }
        //             }
        //         }
        //     });

        // self.table4 = receiver.iter().collect();
        // self.table4.sort_by(|a, b| a.0.value.cmp(&b.0.value));

        // println!(
        //     "Table 4 len: {} ({:.2}%)",
        //     self.table4.len(),
        //     self.table4.len() as f64 / table_size as f64 * 100.0
        // );

        // println!("\nFinal tables:");
        // println!(
        //     "Table 2 len: {} ({:.2}%)",
        //     self.table2.len(),
        //     self.table2.len() as f64 / table_size as f64 * 100.0
        // );
        // println!(
        //     "Table 3 len: {} ({:.2}%)",
        //     self.table3.len(),
        //     self.table3.len() as f64 / table_size as f64 * 100.0
        // );
        // println!(
        //     "Table 4 len: {} ({:.2}%)",
        //     self.table4.len(),
        //     self.table4.len() as f64 / table_size as f64 * 100.0
        // );
        // println!("{:?}\n", self.table1);
        // println!("{:?}\n", self.table2);
        // println!("{:?}\n", self.table3);
        // println!("{:?}", self.table4);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "outdated"]
    fn test_plotting() {
        let k = 12;
        let plot_seed = b"abcdabcdabcdabcdabcdabcdabcdabcd";

        let mut pos1 = PoSpace::new(k, plot_seed);
        pos1.run_phase_1();
        let mut pos2 = PoSpace::new(k, plot_seed);
        pos2.run_phase_1();

        for tuple in pos1.table2.iter().zip(pos2.table2.iter()) {
            assert_eq!(tuple.0 .0, tuple.1 .0);
        }

        for tuple in pos1.table3.iter().zip(pos2.table3.iter()) {
            assert_eq!(tuple.0 .0, tuple.1 .0);
        }

        for tuple in pos1.table4.iter().zip(pos2.table4.iter()) {
            assert_eq!(tuple.0 .0, tuple.1 .0);
        }
    }
}
