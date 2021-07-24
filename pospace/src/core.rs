use crossbeam::channel::bounded;
use log::{debug, error, info, warn};
use rayon::prelude::*;
use std::{
    fs::{create_dir_all, remove_dir_all},
    path::Path,
};

use crate::storage::{deserialize, PlotEntry};
use crate::{
    bits::BitsWrapper,
    constants::{PARAM_B, PARAM_BC, PARAM_C, PARAM_EXT, PARAM_M},
    f1_calculator::F1Calculator,
    fx_calculator::FXCalculator,
    storage::{
        sort_table_on_disk, store_table1_part, ENTRIES_PER_CHUNK, TABLE1_SERIALIZED_ENTRY_SIZE,
    },
};
use std::fs::{read_dir, File};
use std::io::Read;

use crate::bits::to_bits;
use crate::table_final_filename_format;
use std::cmp::min;

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

        let data_path = Path::new("data");

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
                buffer.push(PlotEntry {
                    fx: data.0.value,
                    x: Some(data.1.value),
                    position: None,
                    offset: None,
                    collate: None,
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
                    store_table1_part(&buffer, data_path, counter);
                    counter += 1;
                    buffer.clear();
                }
            }

            if buffer.len() > 0 {
                store_table1_part(&buffer, data_path, counter);
            }
        });

        info!("Table 1 raw data written");
        info!("Starting to sort table 1 on disk ...");

        sort_table_on_disk::<PlotEntry>(1, data_path, ENTRIES_PER_CHUNK);

        info!("Table 1 sorted on disk");

        // Load part of table 1 in memory
        info!("Calculating table 2 ...");

        let mut file =
            File::open(data_path.join(format!(table_final_filename_format!(), 1))).unwrap();
        let number_of_entries = min(ENTRIES_PER_CHUNK, 2usize.pow(self.k as u32));
        let mut buffer = vec![0u8; number_of_entries * *TABLE1_SERIALIZED_ENTRY_SIZE];
        file.read_exact(&mut buffer).unwrap();
        let data: Vec<PlotEntry> = deserialize(&buffer);

        let mut counter = 0;
        let mut bucket = 0;
        let mut left_bucket = Vec::new();
        let mut right_bucket = Vec::new();

        for left_entry in data {
            // debug!("{:?}", left_entry);
            let y_bucket = left_entry.fx / PARAM_BC;
            if y_bucket == bucket {
                left_bucket.push(left_entry);
            } else if y_bucket == bucket + 1 {
                right_bucket.push(left_entry);
            } else {
                if !left_bucket.is_empty() && !right_bucket.is_empty() {
                    // Check for matches
                    let matches = self.fx_calculator.find_matches(&left_bucket, &right_bucket);
                    // if matches.len() >= 10_000 {
                    //     error!("Too many matches: {} is >= 10,000", matches.len());
                    //     panic!("Too many matches");
                    // }
                    // debug!("{} matches found", matches.len());
                    // if matches.len() >= 3 {
                    //     debug!("{:?}", matches[0]);
                    //     debug!("{:?}", matches[1]);
                    //     debug!("{:?}", matches[2]);
                    // }
                    counter += matches.len();

                    // for match_item in matches {
                    // let left_entry = &left_bucket[match_item.left_index];
                    // let right_entry = &right_bucket[match_item.right_index];
                    //
                    // let f_output = self.fx_calculator.calculate_fn(
                    //     &[
                    //         &to_bits(left_entry.x.unwrap(), self.k),
                    //         &to_bits(right_entry.x.unwrap(), self.k),
                    //     ],
                    //     &to_bits(left_entry.fx, self.k + PARAM_EXT),
                    // );
                    // }
                }

                if y_bucket == bucket + 2 {
                    bucket += 1;
                    left_bucket = right_bucket.clone();
                    right_bucket.clear();
                    right_bucket.push(left_entry);
                } else {
                    bucket = y_bucket;
                    left_bucket.clear();
                    left_bucket.push(left_entry);
                    right_bucket.clear();
                }
            }
        }

        info!(
            "{} matches found in total ({:.3}%)",
            counter,
            (counter as f64 / table_size as f64) * 100.0
        );

        // Check for matchs in the window

        // Store matchs in table 2

        // Calculate C3 collate value

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

    // #[test]
    // #[ignore = "outdated"]
    // fn test_plotting() {
    //     let k = 12;
    //     let plot_seed = b"abcdabcdabcdabcdabcdabcdabcdabcd";
    //
    //     let mut pos1 = PoSpace::new(k, plot_seed);
    //     pos1.run_phase_1();
    //     let mut pos2 = PoSpace::new(k, plot_seed);
    //     pos2.run_phase_1();
    // }
}
