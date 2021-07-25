use crossbeam::channel::bounded;
use log::{debug, error, info, warn};
use rayon::prelude::*;
use std::{
    fs::{create_dir_all, remove_dir_all},
    path::Path,
};

use crate::storage::{deserialize, plotentry_size, store_raw_table_part, PlotEntry};
use crate::{
    bits::BitsWrapper,
    constants::{PARAM_B, PARAM_BC, PARAM_C, PARAM_EXT, PARAM_M},
    f1_calculator::F1Calculator,
    fx_calculator::FxCalculator,
    storage::{sort_table_on_disk, store_table_part, ENTRIES_PER_CHUNK},
};
use std::fs::{read_dir, File};
use std::io::Read;

use crate::bits::{from_bits, to_bits};
use crate::table_final_filename_format;
use bincode::serialized_size;
use bitvec::view::BitView;
use std::cmp::min;

const NUMBER_OF_TABLES: usize = 7;

#[derive(Debug)]
pub struct PoSpace {
    plot_seed: Vec<u8>,
    k: usize,
    f1_calculator: F1Calculator,
}

impl PoSpace {
    pub fn new(k: usize, plot_seed: &[u8]) -> Self {
        PoSpace {
            plot_seed: plot_seed.to_vec(),
            k,
            f1_calculator: F1Calculator::new(k, &plot_seed),
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
                    store_raw_table_part(1, counter, &buffer, data_path);
                    counter += 1;
                    buffer.clear();
                }
            }

            if buffer.len() > 0 {
                store_raw_table_part(1, counter, &buffer, data_path);
            }
        });

        info!("Table 1 raw data written");

        for table_index in 1..NUMBER_OF_TABLES {
            info!("Starting to sort table {} on disk ...", table_index);
            sort_table_on_disk::<PlotEntry>(table_index, data_path, ENTRIES_PER_CHUNK, self.k);
            info!("Table {} sorted on disk", table_index);

            info!("Calculating table {} ...", table_index + 1);

            let mut file =
                File::open(data_path.join(format!(table_final_filename_format!(), table_index)))
                    .unwrap();
            let number_of_entries = min(ENTRIES_PER_CHUNK, 2usize.pow(self.k as u32));
            let entry_size = plotentry_size(table_index, self.k);

            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).unwrap();
            let data: Vec<PlotEntry> = deserialize(&buffer, entry_size);

            let mut fx_calculator = FxCalculator::new(self.k, table_index + 1);
            let mut match_counter = 0;
            let mut bucket = 0;
            let mut pos = 0;
            let mut left_bucket = Vec::new();
            let mut right_bucket = Vec::new();
            let mut buffer_to_write = Vec::new();

            for mut left_entry in data {
                left_entry.position = Some(pos);

                let y_bucket = left_entry.fx / PARAM_BC;

                if y_bucket == bucket {
                    left_bucket.push(left_entry);
                } else if y_bucket == bucket + 1 {
                    right_bucket.push(left_entry);
                } else {
                    if !left_bucket.is_empty() && !right_bucket.is_empty() {
                        // Check for matches
                        let matches = fx_calculator.find_matches(&left_bucket, &right_bucket);

                        // Sanity check
                        if matches.len() >= 10_000 {
                            error!("Too many matches: {} is >= 10,000", matches.len());
                            panic!("Too many matches: {} is >= 10,000", matches.len());
                        }

                        match_counter += matches.len();

                        for match_item in matches {
                            let left_entry = &left_bucket[match_item.left_index];
                            let right_entry = &right_bucket[match_item.right_index];

                            let (left_metadata, right_metadata) = if table_index == 1 {
                                (
                                    to_bits(left_entry.x.unwrap(), self.k),
                                    to_bits(right_entry.x.unwrap(), self.k),
                                )
                            } else {
                                (
                                    left_entry.collate.as_ref().unwrap().view_bits()
                                        [..collation_size_bits(table_index + 1, self.k)]
                                        .to_bitvec(),
                                    right_entry.collate.as_ref().unwrap().view_bits()
                                        [..collation_size_bits(table_index + 1, self.k)]
                                        .to_bitvec(),
                                )
                            };

                            assert_eq!(
                                left_metadata.len(),
                                collation_size_bits(table_index + 1, self.k)
                            );
                            assert_eq!(
                                right_metadata.len(),
                                collation_size_bits(table_index + 1, self.k)
                            );

                            let mut f_output = fx_calculator.calculate_fn(
                                &to_bits(left_entry.fx, self.k + PARAM_EXT),
                                &left_metadata,
                                &right_metadata,
                            );

                            assert_eq!(
                                f_output.1.len(),
                                collation_size_bits(table_index + 2, self.k)
                            );

                            // f_output.1.force_align();

                            buffer_to_write.push(PlotEntry {
                                fx: from_bits(&f_output.0),
                                x: None,
                                position: Some(left_entry.position.unwrap()),
                                offset: Some(
                                    right_entry.position.unwrap() - left_entry.position.unwrap(),
                                ),
                                collate: Some(f_output.1.as_raw_slice().to_vec()),
                            })
                        }
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

                pos += 1;
            }

            info!(
                "{} matches found in total ({:.3}%) for table {}",
                match_counter,
                (match_counter as f64 / table_size as f64) * 100.0,
                table_index + 1
            );

            let size = serialized_size(&buffer_to_write[0]);

            info!("Writing raw table {} to disk", table_index + 1);
            // TODO: make multipart writes
            store_raw_table_part(table_index + 1, 1, &buffer_to_write, data_path);
            info!("Table {} raw data written", table_index + 1);
        }
    }
}

/// Size in bits
pub fn collation_size_bits(table_index: usize, k: usize) -> usize {
    k * match table_index {
        2 => 1,
        3 | 7 => 2,
        6 => 3,
        4 | 5 => 4,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
