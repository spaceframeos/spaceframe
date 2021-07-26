use crate::storage::{deserialize, plotentry_size, serialize, store_table_part};
use log::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::fs::{read_dir, remove_file, rename, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::{table_final_filename_format, table_sorted_filename_format};
use std::collections::VecDeque;

pub fn sort_table_part<T>(path: &Path, table_index: usize, part_index: usize, k: usize) -> PathBuf
where
    T: Serialize + DeserializeOwned + Ord + Debug,
{
    info!("[Table {}] Sorting part {} ...", table_index, part_index);
    let mut buffer = Vec::new();
    let mut file = File::open(&path).unwrap();
    file.read_to_end(&mut buffer).unwrap();
    let mut entries = deserialize::<Vec<T>, T>(&buffer, plotentry_size(table_index, k));

    entries.sort_unstable();

    let out_path = path.parent().unwrap().join(format!(
        table_sorted_filename_format!(),
        table_index, part_index
    ));

    store_table_part(&entries, &out_path);
    info!("[Table {}] Part {} sorted", table_index, part_index);
    out_path
}

pub fn sort_table_on_disk<T>(table_index: usize, path: &Path, entries_per_chunk: usize, k: usize)
where
    T: Serialize + DeserializeOwned + Ord + Debug,
{
    // Sort each bucket
    let mut chunks_count = 0;
    let mut parts = Vec::new();

    // Sort individual table parts
    for (index, entry) in read_dir(path)
        .unwrap()
        .filter_map(Result::ok)
        .map(|x| x.path())
        .filter(|e| {
            e.is_file()
                && e.file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with(format!("table{}_raw_", table_index).as_str())
        })
        .enumerate()
    {
        let part_path = sort_table_part::<T>(&entry, table_index, index + 1, k);
        parts.push(part_path);
        chunks_count += 1;
    }

    // K-Way Merge sort

    if chunks_count > 1 {
        info!("[Table {}] K-Way merging ...", table_index);

        let mut state = KWayMerge::<T>::new(
            &parts,
            plotentry_size(table_index, k),
            entries_per_chunk,
            &path.join(format!(table_final_filename_format!(), table_index)),
            table_index,
        );

        while state.run_iteration() != KWayMergeState::Done {}

        info!("[Table {}] K-Way merge done", table_index);
    } else {
        rename(
            path.join(format!(table_sorted_filename_format!(), table_index, 1)),
            path.join(format!(table_final_filename_format!(), table_index)),
        )
        .unwrap();
    }

    info!("[Table {}] Cleaning intermediate files ...", table_index);
    read_dir(path)
        .unwrap()
        .filter_map(Result::ok)
        .map(|x| x.path())
        .filter(|e| {
            let filename = e.file_name().unwrap().to_str().unwrap();
            return e.is_file()
                && (filename.starts_with(format!("table{}_raw_", table_index).as_str())
                    || filename.starts_with(format!("table{}_sorted_", table_index).as_str()));
        })
        .for_each(|f| remove_file(f).unwrap());
}

#[derive(Debug, PartialEq)]
enum KWayMergeState {
    Success,
    Done,
}

#[derive(Debug)]
struct KWayMerge<T> {
    entries_per_chunk: usize,
    output_file: File,
    chunks: Vec<MergeChunk<T>>,
    output: Vec<T>,
    iter_count: usize,
    item_count: usize,
    table_index: usize,
}

impl<T> KWayMerge<T>
where
    T: Serialize + DeserializeOwned + Ord + Debug,
{
    pub fn new(
        paths: &[PathBuf],
        entry_size: usize,
        entries_per_chunk: usize,
        output_file_path: &Path,
        table_index: usize,
    ) -> Self {
        let chunk_size = entries_per_chunk / (paths.len() - 1) * entry_size;
        let mut state = Self {
            entries_per_chunk,
            chunks: Vec::new(),
            output: Vec::new(),
            iter_count: 0,
            item_count: 0,
            output_file: File::create(output_file_path).unwrap(),
            table_index,
        };

        let mut id_counter = 1;

        for path in paths {
            let file = File::open(path).unwrap();
            let file_size = file.metadata().unwrap().len() as usize;
            let merge_chunk = MergeChunk {
                id: id_counter,
                file,
                entry_size,
                total_size: file_size,
                remaining_size: file_size,
                content: VecDeque::new(),
                chunk_size,
            };
            state.chunks.push(merge_chunk);
            id_counter += 1;
        }

        state
    }

    pub fn run_iteration(&mut self) -> KWayMergeState {
        // Load new data into chunks if they are empty
        for chunk in self.chunks.iter_mut() {
            // Refill chunk
            chunk.refill();
        }

        // Find the min
        let min = self.find_min_chunk().unwrap();
        let min_chunk = &mut self.chunks[min];

        // Move the minimum value to the output vec
        // Delete the minimum from the chunk (increase the index)
        self.output.push(min_chunk.content.pop_front().unwrap());

        // Write output if it is full
        if self.output.len() >= self.entries_per_chunk {
            self.write_output();
            self.output.clear();
        }

        // Keeping only unfinished chunks
        self.chunks.retain(|x| !x.is_done());

        if self.chunks.len() == 0 {
            self.write_output();
            info!(
                "[Table {}] Final part {} written",
                self.table_index, self.iter_count
            );
            return KWayMergeState::Done;
        }

        return KWayMergeState::Success;
    }

    pub fn find_min_chunk(&self) -> Result<usize, ChunkError> {
        if self.chunks.iter().all(|c| c.content.get(0).is_some()) {
            Ok(self
                .chunks
                .iter()
                .map(|c| &c.content[0])
                .collect::<Vec<&T>>()
                .iter()
                .enumerate()
                .min_by_key(|&(_, x)| x)
                .unwrap()
                .0)
        } else {
            return Err(ChunkError::EmptyChunksWhileFetchingMininum);
        }
    }

    fn write_output(&mut self) {
        self.iter_count += 1;
        self.item_count += self.output.len();
        if !self.output.is_empty() {
            let bin_data = serialize(&self.output);
            self.output_file.write_all(&bin_data).unwrap();
        }
    }
}

#[derive(Debug)]
struct MergeChunk<T> {
    id: u32,
    file: File, // TODO don't keep the file open to prevent "Too many open files" error
    content: VecDeque<T>,
    entry_size: usize,
    total_size: usize,
    chunk_size: usize,
    remaining_size: usize,
}

impl<T> MergeChunk<T>
where
    T: Serialize + DeserializeOwned + Ord,
{
    pub fn refill(&mut self) {
        if self.content.len() == 0 && self.remaining_size > 0 {
            let amount;
            let mut buffer;
            if self.remaining_size > self.chunk_size {
                // Read only 1 chunk
                buffer = vec![0u8; self.chunk_size];
                self.file.read_exact(&mut buffer).unwrap();
                amount = self.chunk_size;
            } else {
                // Read to the end
                buffer = Vec::new();
                amount = self.file.read_to_end(&mut buffer).unwrap();
            }

            self.remaining_size -= amount;

            // Deserilalize entries
            let entries = deserialize(&buffer, self.entry_size);
            self.content = entries;
        }
    }

    pub fn is_done(&self) -> bool {
        return self.content.len() == 0 && self.remaining_size == 0;
    }
}

#[derive(Debug)]
enum ChunkError {
    EmptyChunksWhileFetchingMininum,
}
