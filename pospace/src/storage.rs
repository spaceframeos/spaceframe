use glob::glob;
use std::{
    collections::VecDeque,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::bits::BitsWrapper;
use serde::{Deserialize, Serialize};

// pub const ENTRIES_PER_CHUNK: usize = 20_000;

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

pub fn store_table1_part(buffer: &[Table1Entry], folder: &str, index: usize, suffix: Option<&str>) {
    let mut new_file = File::create(Path::new(folder).join(format!(
        "table1_{}{}",
        index,
        suffix.or(Some("")).unwrap()
    )))
    .unwrap();
    let bin_data = serialize(buffer);
    new_file.write_all(&bin_data).unwrap();
}

pub fn serialize(buffer: &[Table1Entry]) -> Vec<u8> {
    buffer
        .iter()
        .flat_map(|entry| bincode::serialize(entry).unwrap())
        .collect::<Vec<u8>>()
}

pub fn sort_table_part(path: &Path) -> Option<PathBuf> {
    if path.is_file() {
        let mut buffer = Vec::new();
        let mut file = File::open(&path).unwrap();
        file.read_to_end(&mut buffer).unwrap();
        let mut entries = buffer
            .chunks(*TABLE1_SERIALIZED_ENTRY_SIZE)
            .map(|chunk| {
                return bincode::deserialize(&chunk).unwrap();
            })
            .collect::<Vec<Table1Entry>>();

        entries.sort();

        let out_path = path.parent().unwrap().join(format!(
            "{}_sorted",
            String::from(path.file_name().unwrap().to_str().unwrap())
        ));

        let mut sorted_file = File::create(&out_path).unwrap();
        let bin_data = serialize(&entries);
        sorted_file.write_all(&bin_data).unwrap();

        return Some(out_path);
    }
    return None;
}

pub fn sort_table(tables_folder: &str, table_pattern: &str, entries_per_chunk: usize) {
    // Sort each bucket
    let mut chunks_count = 0;
    let mut parts = Vec::new();

    // Sort individual table parts
    for entry in glob(table_pattern).unwrap().filter_map(Result::ok) {
        sort_table_part(&entry).map(|path| {
            chunks_count += 1;
            parts.push(path);
        });
    }

    // K-Way Merge sort

    if chunks_count > 1 {
        println!("K-Way merging for table 1 ...");

        let mut state = KWayMerge::new(
            &parts,
            *TABLE1_SERIALIZED_ENTRY_SIZE,
            entries_per_chunk,
            tables_folder,
        );

        while state.run_iteration() != KWayMergeState::Done {}

        println!("K-Way merge done");

        println!("{} final entries written", state.item_count);
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
    output_folder: String,
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
        output_folder: &str,
    ) -> Self {
        let chunk_size = entries_per_chunk / (paths.len() - 1) * entry_size;
        let mut state = Self {
            entries_per_chunk,
            chunks: Vec::new(),
            output: Vec::new(),
            iter_count: 0,
            item_count: 0,
            output_folder: String::from(output_folder),
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
    file: File,
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
                // Read 1 chunk
                buffer = vec![0u8; self.chunk_size];
                amount = self.file.read(&mut buffer).unwrap();
            } else {
                // Read to end
                buffer = Vec::new();
                amount = self.file.read_to_end(&mut buffer).unwrap();
            }

            self.remaining_size -= amount;

            // Deserilalize entries
            let entries = buffer
                .chunks(*TABLE1_SERIALIZED_ENTRY_SIZE as usize)
                .filter_map(|chunk| {
                    if chunk.iter().all(|c| c == &0u8) {
                        return None;
                    } else {
                        return Some(bincode::deserialize(&chunk).unwrap());
                    }
                })
                .collect::<VecDeque<Table1Entry>>();
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
