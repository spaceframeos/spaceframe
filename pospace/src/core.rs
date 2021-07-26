use crossbeam::channel::bounded;
use log::*;
use rayon::prelude::*;
use std::{
    fs::{create_dir_all, remove_dir_all},
    path::Path,
};

use crate::storage::{deserialize, plotentry_size, store_raw_table_part, PlotEntry};
use crate::{
    bits::BitsWrapper,
    constants::{PARAM_BC, PARAM_EXT},
    f1_calculator::F1Calculator,
    fx_calculator::FxCalculator,
    storage::ENTRIES_PER_CHUNK,
    BitsSlice,
};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use crate::bits::{from_bits, to_bits};
use crate::sort::sort_table_on_disk;
use crate::table_final_filename_format;
use bitvec::view::BitView;
use std::path::PathBuf;

const NUMBER_OF_TABLES: usize = 7;

pub type PlotSeed = [u8; 32];

#[derive(Debug, Clone)]
pub struct PoSpace {
    pub plot_seed: PlotSeed,
    pub k: usize,
    f1_calculator: F1Calculator,
    data_path: PathBuf,
}

impl PoSpace {
    pub fn new(k: usize, plot_seed: PlotSeed, data_path: &Path) -> Self {
        PoSpace {
            plot_seed,
            k,
            f1_calculator: F1Calculator::new(k, plot_seed),
            data_path: data_path.to_owned(),
        }
    }

    pub fn run_phase_1(&mut self) {
        // Clear data folder
        match remove_dir_all(&self.data_path) {
            Ok(_) => {
                info!("Cleaning data folder");
            }
            Err(e) => {
                warn!("Cannot clean data folder: {}", e);
            }
        }
        create_dir_all(&self.data_path).ok();
        info!("Data dir cleaned");

        let table_size = 1u64 << self.k;

        info!("[Table 1] Calculating buckets ...");

        rayon::scope(|s| {
            let (sender, receiver) = bounded(*ENTRIES_PER_CHUNK);

            s.spawn(|_| {
                (0..table_size)
                    .into_par_iter()
                    .for_each_with(sender, |s, x| {
                        let x_wrapped = BitsWrapper::from(x, self.k);
                        let fx = self.f1_calculator.calculate_f1(&x_wrapped);
                        s.send((BitsWrapper::new(fx), x_wrapped)).unwrap();
                    });
            });

            let mut buffer = Vec::new();
            let mut counter = 0;

            while let Ok(data) = receiver.recv() {
                buffer.push(PlotEntry {
                    fx: data.0.value,
                    metadata: Some(data.1.bits.as_raw_slice().to_vec()),
                    position: None,
                    offset: None,
                });

                if buffer.len() % (1024 * 1024 * 4) == 0 {
                    info!(
                        "[Table 1] Calculating progess: {:.3}%",
                        (buffer.len() + counter * *ENTRIES_PER_CHUNK) as f64
                            / (table_size as usize) as f64
                            * 100 as f64
                    );
                }

                if buffer.len() == *ENTRIES_PER_CHUNK {
                    counter += 1;
                    info!("[Table 1] Wrinting part {} to disk ...", counter);
                    store_raw_table_part(1, counter, &buffer, &self.data_path);
                    buffer.clear();
                }
            }

            if buffer.len() > 0 {
                info!("[Table 1] Wrinting part {} to disk ...", counter);
                store_raw_table_part(1, counter, &buffer, &self.data_path);
            }
        });

        info!("[Table 1] Sorting table on disk ...");
        sort_table_on_disk::<PlotEntry>(1, &self.data_path, *ENTRIES_PER_CHUNK, self.k);
        info!("[Table 1] Sorting table on disk done");
        info!("[Table 1] Table ready");

        for table_index in 2..=NUMBER_OF_TABLES {
            info!("[Table {}] Calculating buckets ...", table_index);
            let mut file = File::open(
                self.data_path
                    .join(format!(table_final_filename_format!(), table_index - 1)),
            )
            .unwrap();
            let file_size = file.metadata().unwrap().len() as usize;
            let entry_size = plotentry_size(table_index - 1, self.k);
            let mut remaining_size = file_size;

            assert_eq!(file_size % entry_size, 0);

            let mut fx_calculator = FxCalculator::new(self.k, table_index);
            let mut match_counter = 0;
            let mut bucket = 0;
            let mut pos = 0;
            let mut part = 0;
            let mut left_bucket = Vec::new();
            let mut right_bucket = Vec::new();
            let mut buffer_to_write = Vec::new();

            while remaining_size > 0 {
                let mut buffer;
                if remaining_size > *ENTRIES_PER_CHUNK * entry_size {
                    buffer = vec![0; *ENTRIES_PER_CHUNK * entry_size];
                    file.read_exact(&mut buffer).unwrap();
                    remaining_size -= *ENTRIES_PER_CHUNK * entry_size;
                } else {
                    buffer = Vec::new();
                    let amount = file.read_to_end(&mut buffer).unwrap();
                    remaining_size -= amount;
                }

                let entries: Vec<PlotEntry> = deserialize(&buffer, entry_size);

                for mut left_entry in entries {
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

                                let (left_metadata, right_metadata) = (
                                    left_entry.metadata.as_ref().unwrap().view_bits()
                                        [..collation_size_bits(table_index, self.k)]
                                        .to_bitvec(),
                                    right_entry.metadata.as_ref().unwrap().view_bits()
                                        [..collation_size_bits(table_index, self.k)]
                                        .to_bitvec(),
                                );

                                assert_eq!(
                                    left_metadata.len(),
                                    collation_size_bits(table_index, self.k)
                                );
                                assert_eq!(
                                    right_metadata.len(),
                                    collation_size_bits(table_index, self.k)
                                );

                                let f_output = fx_calculator.calculate_fn(
                                    &to_bits(left_entry.fx, self.k + PARAM_EXT),
                                    &left_metadata,
                                    &right_metadata,
                                );

                                assert_eq!(
                                    f_output.1.len(),
                                    collation_size_bits(table_index + 1, self.k)
                                );

                                buffer_to_write.push(PlotEntry {
                                    fx: from_bits(&f_output.0),
                                    metadata: Some(f_output.1.as_raw_slice().to_vec()),
                                    position: Some(left_entry.position.unwrap()),
                                    offset: Some(
                                        right_entry.position.unwrap()
                                            - left_entry.position.unwrap(),
                                    ),
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

                    if match_counter >= (table_size * 2) as usize {
                        warn!("Too many match, skipping...");
                        break;
                    }
                }

                part += 1;

                if !buffer_to_write.is_empty() {
                    info!("[Table {}] Writing part {} to disk", table_index, part);
                    store_raw_table_part(table_index, part, &buffer_to_write, &self.data_path);
                    buffer_to_write.clear();
                }

                if match_counter >= (table_size * 2) as usize {
                    break;
                }
            }

            info!(
                "[Table {}] {} matches found ({:.3}% of table 1 size)",
                table_index,
                match_counter,
                (match_counter as f64 / table_size as f64) * 100.0
            );

            info!("[Table {}] Sorting table on disk ...", table_index);
            sort_table_on_disk::<PlotEntry>(
                table_index,
                &self.data_path,
                *ENTRIES_PER_CHUNK,
                self.k,
            );
            info!("[Table {}] Sorting table on disk done", table_index);
            info!("[Table {}] Table ready", table_index);
        }
    }

    pub fn find_xvalues_from_target(&self, target: &BitsSlice) -> Vec<Vec<PlotEntry>> {
        assert_eq!(target.len(), self.k);

        let mut last_table = File::open(
            self.data_path
                .join(format!(table_final_filename_format!(), 7)),
        )
        .unwrap();
        let file_size = last_table.metadata().unwrap().len() as usize;
        let entry_size = plotentry_size(7, self.k);
        let mut remaining_size = file_size;
        let mut proofs = Vec::new();

        while remaining_size > 0 {
            let mut buffer;
            if remaining_size > *ENTRIES_PER_CHUNK * entry_size {
                buffer = vec![0; *ENTRIES_PER_CHUNK * entry_size];
                last_table.read_exact(&mut buffer).unwrap();
                remaining_size -= *ENTRIES_PER_CHUNK * entry_size;
            } else {
                buffer = Vec::new();
                let amount = last_table.read_to_end(&mut buffer).unwrap();
                remaining_size -= amount;
            }

            let entries: Vec<PlotEntry> = deserialize(&buffer, entry_size);

            let potential_proof_entries: Vec<PlotEntry> = entries
                .into_par_iter()
                .filter(|entry| to_bits(entry.fx, self.k + PARAM_EXT)[..self.k] == target)
                .collect();

            for table7_entry in potential_proof_entries {
                let mut entries_buffer = Vec::new();
                let mut temp_buffer = Vec::new();
                entries_buffer.push(table7_entry);

                for i in (1..=6).rev() {
                    for entry in &entries_buffer {
                        let pos = entry.position.unwrap();
                        let offset = entry.offset.unwrap();
                        let entry_size = plotentry_size(i, self.k);

                        let mut table_i = File::open(
                            self.data_path
                                .join(format!(table_final_filename_format!(), i)),
                        )
                        .unwrap();

                        let mut buffer = vec![0u8; entry_size];

                        table_i
                            .seek(SeekFrom::Start(pos * entry_size as u64))
                            .unwrap();
                        table_i.read_exact(&mut buffer).unwrap();
                        let left_entry: PlotEntry = bincode::deserialize(&buffer).unwrap();

                        table_i
                            .seek(SeekFrom::Start((pos + offset) * entry_size as u64))
                            .unwrap();
                        table_i.read_exact(&mut buffer).unwrap();
                        let right_entry: PlotEntry = bincode::deserialize(&buffer).unwrap();

                        temp_buffer.push(left_entry);
                        temp_buffer.push(right_entry);
                    }
                    entries_buffer.clear();
                    entries_buffer.append(&mut temp_buffer);

                    let str = entries_buffer
                        .iter()
                        .map(|e| e.fx.to_string())
                        .collect::<Vec<String>>()
                        .join(", ");
                }
                proofs.push(entries_buffer);
            }
        }
        return proofs;
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
    // use super::*;
}
