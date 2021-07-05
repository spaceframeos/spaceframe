use log::info;
use std::{
    collections::VecDeque,
    fs::{read_dir, File},
    io::{Read, Write},
    iter::FromIterator,
    path::{Path, PathBuf},
};

use crate::bits::BitsWrapper;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// 1 GB per chunk
pub const ENTRIES_PER_CHUNK: usize = 65_536 * 1024;

lazy_static! {
    pub static ref TABLE1_SERIALIZED_ENTRY_SIZE: usize = bincode::serialized_size(&Table1Entry {
        x: u64::MAX,
        y: u64::MAX
    })
    .unwrap() as usize;
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Table1Entry {
    pub x: u64,
    pub y: u64,
}

impl Ord for Table1Entry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.y.cmp(&other.y)
    }
}

impl PartialOrd for Table1Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

pub struct PlotEntry {
    pub y: BitsWrapper,
    pub pos: u64,
    pub offset: u64,
}

pub fn store_table_part<T>(buffer: &[T], path: &Path)
where
    T: Serialize,
{
    let mut new_file = File::create(path).unwrap();
    let bin_data = serialize(buffer);
    new_file.write_all(&bin_data).unwrap();
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

pub fn deserialize<'de, O, I>(buffer: &'de [u8]) -> O
where
    O: FromIterator<I>,
    I: Deserialize<'de>,
{
    buffer
        .chunks(*TABLE1_SERIALIZED_ENTRY_SIZE)
        .map(|chunk| bincode::deserialize(&chunk).unwrap())
        .collect::<O>()
}

pub fn sort_table_part<T>(path: &Path, table_index: usize, part_index: usize) -> PathBuf
where
    T: Serialize + DeserializeOwned + Ord,
{
    let mut buffer = Vec::new();
    let mut file = File::open(&path).unwrap();
    file.read_to_end(&mut buffer).unwrap();
    let mut entries = deserialize::<Vec<T>, T>(&buffer);

    entries.sort();

    let out_path = path
        .parent()
        .unwrap()
        .join(format!("table{}_sorted_{}", table_index, part_index));

    store_table_part(&entries, &out_path);
    out_path
}

pub fn sort_table_on_disk<T>(table_index: usize, path: &Path, entries_per_chunk: usize)
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
        .enumerate()
    {
        if entry.is_file()
            && entry
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with(format!("table{}_raw_", table_index).as_str())
        {
            let part_path = sort_table_part::<T>(&entry, table_index, index + 1);
            parts.push(part_path);
            chunks_count += 1;
        }
    }

    // K-Way Merge sort

    if chunks_count > 1 {
        info!("K-Way merging for table 1 ...");

        let mut state = KWayMerge::new(
            &parts,
            *TABLE1_SERIALIZED_ENTRY_SIZE,
            entries_per_chunk,
            path,
        );

        while state.run_iteration() != KWayMergeState::Done {}

        info!("K-Way merge done");

        info!("{} final entries written", state.item_count);
    } else {
        // TODO rename file to final
    }
}

#[derive(Debug, PartialEq)]
enum KWayMergeState {
    Success,
    Done,
}

#[derive(Debug)]
struct KWayMerge {
    entries_per_chunk: usize,
    output_folder: PathBuf,
    chunks: Vec<MergeChunk>,
    output: Vec<Table1Entry>,
    iter_count: usize,
    item_count: usize,
    output_file: File,
}

impl KWayMerge {
    pub fn new(
        paths: &[PathBuf],
        entry_size: usize,
        entries_per_chunk: usize,
        output_folder: &Path,
    ) -> Self {
        let chunk_size = entries_per_chunk / (paths.len() - 1) * entry_size;
        let mut state = Self {
            entries_per_chunk,
            chunks: Vec::new(),
            output: Vec::new(),
            iter_count: 0,
            item_count: 0,
            output_folder: output_folder.to_owned(),
            output_file: File::create(Path::new(output_folder).join("table1_final")).unwrap(),
        };

        let mut id_counter = 1;

        for path in paths {
            let file = File::open(path).unwrap();
            let file_size = file.metadata().unwrap().len() as usize;
            let merge_chunk = MergeChunk {
                id: id_counter,
                file,
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
        self.output.push(min_chunk.content[0]);

        // Delete the minimum from the chunk (increase the index)
        min_chunk.content.pop_front();

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
                .map(|c| c.content[0])
                .collect::<Vec<Table1Entry>>()
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
        let bin_data = serialize(&self.output);
        self.output_file.write_all(&bin_data).unwrap();
    }
}

#[derive(Debug)]
struct MergeChunk {
    id: u32,
    file: File, // TODO don't keep the file open to prevent "Too many open files" error
    content: VecDeque<Table1Entry>,
    total_size: usize,
    chunk_size: usize,
    remaining_size: usize,
}

impl MergeChunk {
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
            let entries = deserialize(&buffer);
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

#[cfg(test)]
mod tests {

    use tempdir::TempDir;

    use super::*;

    #[test]
    fn test_store_table_part_table1() {
        let dir = TempDir::new("spaceframe_test_data").unwrap();
        let test_data = vec![Table1Entry { x: 2, y: 3 }, Table1Entry { x: 6, y: 1 }];
        let path = dir.path().join("store_table_1");
        store_table_part(&test_data, &path);

        let mut verify_buffer = Vec::new();
        File::open(&path)
            .unwrap()
            .read_to_end(&mut verify_buffer)
            .unwrap();
        let verify_data: Vec<Table1Entry> = deserialize(&verify_buffer);

        assert_eq!(test_data, verify_data);
    }
}
