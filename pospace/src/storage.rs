use log::{debug, info};
use std::{
    collections::VecDeque,
    fs::{read_dir, File},
    io::{Read, Write},
    iter::FromIterator,
    path::{Path, PathBuf},
};

use crate::core::collation_size_bits;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fs::rename;

/// 1 GB per chunk
pub const ENTRIES_PER_CHUNK: usize = 65_536 * 1024;

#[macro_export]
macro_rules! table_raw_filename_format {
    () => {
        "table{}_raw_{}"
    };
}

#[macro_export]
macro_rules! table_sorted_filename_format {
    () => {
        "table{}_sorted_{}"
    };
}

#[macro_export]
macro_rules! table_final_filename_format {
    () => {
        "table{}_final"
    };
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct PlotEntry {
    pub fx: u64,
    pub x: Option<u64>,
    pub position: Option<u64>,
    pub offset: Option<u64>,
    pub collate: Option<Vec<u8>>,
}

impl Ord for PlotEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.fx.cmp(&other.fx)
    }
}

impl PartialOrd for PlotEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

pub fn store_table_part<T>(buffer: &[T], path: &Path)
where
    T: Serialize,
{
    let mut new_file = File::create(path).unwrap();
    let bin_data = serialize(buffer);
    new_file.write_all(&bin_data).unwrap();
}

pub fn store_raw_table_part(
    table_index: usize,
    part_index: usize,
    buffer: &[PlotEntry],
    path: &Path,
) {
    store_table_part(
        buffer,
        &path.join(format!(
            table_raw_filename_format!(),
            table_index, part_index
        )),
    );
}

pub fn serialize<T>(buffer: &[T]) -> Vec<u8>
where
    T: Serialize,
{
    buffer
        .iter()
        .flat_map(|entry| bincode::serialize(entry).unwrap())
        .collect::<Vec<u8>>()
}

pub fn deserialize<'de, O, I>(buffer: &'de [u8], entry_size: usize) -> O
where
    O: FromIterator<I>,
    I: Deserialize<'de>,
{
    buffer
        .chunks(entry_size)
        .map(|chunk| bincode::deserialize(&chunk).unwrap())
        .collect::<O>()
}

pub fn sort_table_part<T>(path: &Path, table_index: usize, part_index: usize, k: usize) -> PathBuf
where
    T: Serialize + DeserializeOwned + Ord,
{
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
    out_path
}

pub fn sort_table_on_disk<T>(table_index: usize, path: &Path, entries_per_chunk: usize, k: usize)
where
    T: Serialize + DeserializeOwned + Ord,
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
        info!("K-Way merging for table {} ...", table_index);

        let mut state = KWayMerge::<T>::new(
            &parts,
            plotentry_size(table_index, k),
            entries_per_chunk,
            &path.join(format!(table_final_filename_format!(), table_index)),
        );

        while state.run_iteration() != KWayMergeState::Done {}

        info!("K-Way merge done");

        info!(
            "{} final entries written for table {}",
            state.item_count, table_index
        );
    } else {
        rename(
            path.join(format!(table_sorted_filename_format!(), table_index, 1)),
            path.join(format!(table_final_filename_format!(), table_index)),
        )
        .unwrap();
    }

    // TODO: clean intermediate files
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
}

impl<T> KWayMerge<T>
where
    T: Serialize + DeserializeOwned + Ord,
{
    pub fn new(
        paths: &[PathBuf],
        entry_size: usize,
        entries_per_chunk: usize,
        output_file_path: &Path,
    ) -> Self {
        let chunk_size = entries_per_chunk / (paths.len() - 1) * entry_size;
        let mut state = Self {
            entries_per_chunk,
            chunks: Vec::new(),
            output: Vec::new(),
            iter_count: 0,
            item_count: 0,
            output_file: File::create(output_file_path).unwrap(),
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
                amount = self.file.read(&mut buffer).unwrap();
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

// Size in bytes
pub fn plotentry_size(table_index: usize, k: usize) -> usize {
    return if table_index == 1 {
        4 + 16
    } else {
        4 + 32 + (collation_size_bits(table_index + 1, k) as f64 / 8 as f64).ceil() as usize
    };
}

#[cfg(test)]
mod tests {

    use tempdir::TempDir;

    use super::*;

    #[test]
    fn test_store_table_part_table1() {
        let dir = TempDir::new("spaceframe_test_data").unwrap();
        let test_data = vec![
            PlotEntry {
                fx: 2,
                x: Some(3),
                position: None,
                offset: None,
                collate: None,
            },
            PlotEntry {
                fx: 6,
                x: Some(1),
                position: None,
                offset: None,
                collate: None,
            },
        ];
        let path = dir.path().join("store_table_1");
        store_table_part(&test_data, &path);

        let mut verify_buffer = Vec::new();
        File::open(&path)
            .unwrap()
            .read_to_end(&mut verify_buffer)
            .unwrap();
        let verify_data: Vec<PlotEntry> = deserialize(&verify_buffer, plotentry_size(1, 12));

        assert_eq!(test_data, verify_data);
    }
}
