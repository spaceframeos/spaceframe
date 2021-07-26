use crossbeam_channel::bounded;
use log::*;
use rayon::prelude::*;
use std::{
    fs::{create_dir_all, remove_dir_all},
    path::Path,
};

use crate::storage::{plotentry_size, store_raw_table_part, ChunkReader, PlotEntry};
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
use crate::error::{PoSpaceError, StorageError};
use crate::sort::sort_table_on_disk;
use crate::table_final_filename_format;
use anyhow::{Context, Result};
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
    pub fn new(k: usize, plot_seed: PlotSeed, data_path: &Path) -> Result<Self> {
        if k < 12 || k > 50 {
            return Err(PoSpaceError::InvalidK(k).into());
        }
        Ok(PoSpace {
            plot_seed,
            k,
            f1_calculator: F1Calculator::new(k, plot_seed),
            data_path: data_path.to_owned(),
        })
    }

    pub fn run_phase_1(&mut self) -> Result<()> {
        // Clear data folder
        match remove_dir_all(&self.data_path) {
            Ok(_) => {
                info!("Cleaning data folder");
            }
            Err(e) => {
                warn!("Data folder not cleaned: {}", e);
            }
        }
        create_dir_all(&self.data_path).ok();
        info!("Data dir cleaned and created");

        let table_size = 1u64 << self.k;

        info!("[Table 1] Calculating buckets ...");

        rayon::scope(|s| -> Result<()> {
            let (sender, receiver) = bounded(*ENTRIES_PER_CHUNK);

            s.spawn(|_| {
                (0..table_size)
                    .into_par_iter()
                    .for_each_with(sender, |s, x| {
                        let x_wrapped = BitsWrapper::from(x, self.k);
                        let fx = self.f1_calculator.calculate_f1(&x_wrapped).unwrap();
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
                    store_raw_table_part(1, counter, &buffer, &self.data_path)
                        .context(format!("Failed to store part {} of table 1", counter))?;
                    buffer.clear();
                }
            }

            if buffer.len() > 0 {
                info!("[Table 1] Wrinting part {} to disk ...", counter);
                store_raw_table_part(1, counter, &buffer, &self.data_path)
                    .context(format!("Failed to store part {} of table 1", counter))?;
            }

            Ok(())
        })?;

        info!("[Table 1] Sorting table on disk ...");
        sort_table_on_disk(1, &self.data_path, *ENTRIES_PER_CHUNK, self.k)
            .context(format!("Could not sort table {} on disk", 1))?;
        info!("[Table 1] Sorting table on disk done");
        info!("[Table 1] Table ready");

        for table_index in 2..=NUMBER_OF_TABLES {
            info!("[Table {}] Calculating buckets ...", table_index);

            let mut chunk_reader = ChunkReader::new(&self.data_path, table_index - 1, self.k)
                .context("Could not create chunk reader")?;

            let mut fx_calculator = FxCalculator::new(self.k, table_index);
            let mut match_counter = 0;
            let mut bucket = 0;
            let mut pos = 0;
            let mut part = 0;
            let mut left_bucket = Vec::new();
            let mut right_bucket = Vec::new();
            let mut buffer_to_write = Vec::new();

            loop {
                match chunk_reader.read_chunk() {
                    Ok(entries) => {
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
                                    let matches =
                                        fx_calculator.find_matches(&left_bucket, &right_bucket);

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
                                            left_entry
                                                .metadata
                                                .as_ref()
                                                .ok_or(PoSpaceError::EmptyMetadata)?
                                                .view_bits()
                                                [..collation_size_bits(table_index, self.k)]
                                                .to_bitvec(),
                                            right_entry
                                                .metadata
                                                .as_ref()
                                                .ok_or(PoSpaceError::EmptyMetadata)?
                                                .view_bits()
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
                                            position: Some(
                                                left_entry
                                                    .position
                                                    .ok_or(PoSpaceError::EmptyPosition)?,
                                            ),
                                            offset: Some(
                                                right_entry
                                                    .position
                                                    .ok_or(PoSpaceError::EmptyPosition)?
                                                    - left_entry
                                                        .position
                                                        .ok_or(PoSpaceError::EmptyPosition)?,
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
                            store_raw_table_part(
                                table_index,
                                part,
                                &buffer_to_write,
                                &self.data_path,
                            )
                            .context(format!(
                                "Failed to store part {} of table {}",
                                part, table_index
                            ))?;
                            buffer_to_write.clear();
                        }

                        if match_counter >= (table_size * 2) as usize {
                            break;
                        }
                    }
                    Err(e) => match e.downcast_ref::<StorageError>() {
                        Some(StorageError::EndOfFile) => break,
                        _ => return Err(e),
                    },
                }
            }

            info!(
                "[Table {}] {} matches found ({:.3}% of table 1 size)",
                table_index,
                match_counter,
                (match_counter as f64 / table_size as f64) * 100.0
            );

            info!("[Table {}] Sorting table on disk ...", table_index);
            sort_table_on_disk(table_index, &self.data_path, *ENTRIES_PER_CHUNK, self.k)
                .context(format!("Could not sort table {} on disk", table_index))?;
            info!("[Table {}] Sorting table on disk done", table_index);
            info!("[Table {}] Table ready", table_index);
        }

        Ok(())
    }

    pub fn find_xvalues_from_target(&self, target: &BitsSlice) -> Result<Vec<Vec<PlotEntry>>> {
        assert_eq!(target.len(), self.k);

        let mut chunk_reader = ChunkReader::new(&self.data_path, 7, self.k)
            .context("Could not create chunk reader")?;
        let mut proofs = Vec::new();

        loop {
            match chunk_reader.read_chunk() {
                Ok(entries) => {
                    let potential_proof_entries: Vec<PlotEntry> = entries
                        .into_par_iter()
                        .filter(|entry| to_bits(entry.fx, self.k + PARAM_EXT)[..self.k] == target)
                        .collect();

                    for table7_entry in potential_proof_entries {
                        let mut entries_buffer = Vec::new();
                        let mut temp_buffer = Vec::new();
                        entries_buffer.push(table7_entry);

                        // Going from table 6 to table 1
                        for i in (1..=6).rev() {
                            for entry in &entries_buffer {
                                let pos = entry.position.ok_or(PoSpaceError::EmptyPosition)?;
                                let offset = entry.offset.ok_or(PoSpaceError::EmptyOffset)?;
                                let entry_size = plotentry_size(i, self.k);

                                let mut table_i = File::open(
                                    self.data_path
                                        .join(format!(table_final_filename_format!(), i)),
                                )?;

                                let mut buffer = vec![0u8; entry_size];

                                // Retrieve left entry
                                table_i
                                    .seek(SeekFrom::Start(pos * entry_size as u64))
                                    .context(format!(
                                        "Could not seek to left entry at position {} in table {}",
                                        pos, i
                                    ))?;
                                table_i.read_exact(&mut buffer).context(format!(
                                    "Could not read left entry at position {} in table {}",
                                    pos, i
                                ))?;
                                let left_entry: PlotEntry = bincode::deserialize(&buffer)
                                    .or(Err(StorageError::DeserializationError))
                                    .context(format!(
                                        "Could not deserialize left entry in table {}",
                                        i
                                    ))?;

                                // Retrieve right entry
                                table_i
                                    .seek(SeekFrom::Start((pos + offset) * entry_size as u64))
                                    .context(format!(
                                        "Could not seek to right entry at position {} with offset {} in table {}",
                                        pos, offset, i
                                    ))?;
                                table_i.read_exact(&mut buffer).context(format!(
                                    "Could not read right entry at position {} with offset {} in table {}",
                                    pos, offset, i
                                ))?;
                                let right_entry: PlotEntry = bincode::deserialize(&buffer)
                                    .or(Err(StorageError::DeserializationError))
                                    .context(format!(
                                        "Could not deserialize right entry in table {}",
                                        i
                                    ))?;

                                temp_buffer.push(left_entry);
                                temp_buffer.push(right_entry);
                            }
                            entries_buffer.clear();
                            entries_buffer.append(&mut temp_buffer);
                        }
                        proofs.push(entries_buffer);
                    }
                }
                Err(e) => match e.downcast_ref::<StorageError>() {
                    Some(StorageError::EndOfFile) => break,
                    _ => return Err(e),
                },
            }
        }
        Ok(proofs)
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

    #[test]
    fn test_invalid_k_too_small() {
        const TEST_K: usize = 11;
        let plot_seed = *b"aaaabbbbccccddddaaaabbbbccccdddd";
        let pos = PoSpace::new(TEST_K, plot_seed, "test_data".as_ref());
        assert!(pos.is_err());
    }

    #[test]
    fn test_invalid_k_too_large() {
        const TEST_K: usize = 51;
        let plot_seed = *b"aaaabbbbccccddddaaaabbbbccccdddd";
        let pos = PoSpace::new(TEST_K, plot_seed, "test_data".as_ref());
        assert!(pos.is_err());
    }

    #[test]
    fn test_valid_k_lower_bound() {
        const TEST_K: usize = 12;
        let plot_seed = *b"aaaabbbbccccddddaaaabbbbccccdddd";
        let pos = PoSpace::new(TEST_K, plot_seed, "test_data".as_ref());
        assert!(pos.is_ok());
    }

    #[test]
    fn test_valid_k_upper_bound() {
        const TEST_K: usize = 50;
        let plot_seed = *b"aaaabbbbccccddddaaaabbbbccccdddd";
        let pos = PoSpace::new(TEST_K, plot_seed, "test_data".as_ref());
        assert!(pos.is_ok());
    }

    #[test]
    fn test_collate_size() {
        const TEST_K: usize = 12;
        assert_eq!(collation_size_bits(1, TEST_K), 0);
        assert_eq!(collation_size_bits(2, TEST_K), TEST_K);
        assert_eq!(collation_size_bits(3, TEST_K), 2 * TEST_K);
        assert_eq!(collation_size_bits(4, TEST_K), 4 * TEST_K);
        assert_eq!(collation_size_bits(5, TEST_K), 4 * TEST_K);
        assert_eq!(collation_size_bits(6, TEST_K), 3 * TEST_K);
        assert_eq!(collation_size_bits(7, TEST_K), 2 * TEST_K);
        assert_eq!(collation_size_bits(8, TEST_K), 0);
    }
}
