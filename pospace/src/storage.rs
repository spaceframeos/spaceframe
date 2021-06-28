use glob::glob;
use std::{
    cmp::max,
    collections::VecDeque,
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use crate::bits::BitsWrapper;
use flate2::{write::DeflateEncoder, Compression};
use serde::{Deserialize, Serialize};

pub const BUCKET_SIZE: usize = 20_000;

lazy_static! {
    static ref TABLE1_SERIALIZED_ENTRY_SIZE: usize = bincode::serialized_size(&Table1Entry {
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

pub fn store_table1_part(buffer: &[Table1Entry], index: usize, suffix: Option<&str>) {
    let mut new_file = File::create(Path::new("data").join(format!(
        "table1_{}{}",
        index,
        suffix.or(Some("")).unwrap()
    )))
    .unwrap();
    let bin_data = buffer
        .iter()
        .flat_map(|entry| return bincode::serialize(entry).unwrap())
        .collect::<Vec<u8>>();
    new_file.write_all(&bin_data).unwrap();
}

pub fn sort_table1() {
    // Sort each bucket
    let mut chunks_count = 0;
    let mut parts = Vec::new();

    // Sort individual table parts
    for entry in glob("data/table1_*").unwrap() {
        match entry {
            Ok(path) => {
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
                    let path = Path::new("data").join(format!(
                        "{}_sorted",
                        String::from(path.file_name().unwrap().to_str().unwrap())
                    ));
                    let mut sorted_file = File::create(&path).unwrap();

                    parts.push(path);

                    let bin_data = entries
                        .iter()
                        .flat_map(|x| bincode::serialize(x).unwrap())
                        .collect::<Vec<u8>>();

                    sorted_file.write_all(&bin_data).unwrap();
                    chunks_count += 1;
                }
            }
            Err(_) => todo!(),
        }
    }

    // K-Way Merge sort

    // Load BUCKET_SIZE / (chunks_count - 1) buckets into RAM
    if chunks_count > 1 {
        println!("K-Way merging for table 1 ...");
        let mut state = KWayMerge::new(&parts, *TABLE1_SERIALIZED_ENTRY_SIZE);
        println!(
            "{:?}",
            state
                .chunks
                .iter()
                .map(|p| p.content.len())
                .collect::<Vec<usize>>()
        );

        loop {
            match state.run_iteration().unwrap() {
                KWayMergeState::Success => {}
                KWayMergeState::Done => break,
            }
        }

        println!("K-Way merge done");

        println!("{} final entries written", state.item_count);

        // println!("State: {:?}", state);
    }
}

#[derive(Debug)]
enum KWayMergeState {
    Success,
    Done,
}

#[derive(Debug)]
struct KWayMerge {
    chunks: Vec<MergeChunk>,
    output: Vec<Table1Entry>,
    iter_count: usize,
    item_count: usize,
}

impl KWayMerge {
    pub fn new(paths: &[PathBuf], entry_size: usize) -> Self {
        let chunk_size = BUCKET_SIZE / (paths.len() - 1) * entry_size;
        println!("Chunk size: {}", chunk_size);
        println!("Nb of chunks: {}", paths.len() - 1);
        println!("Entry size: {}", entry_size);
        println!("Entry per chunk: {}", BUCKET_SIZE);

        let mut state = Self {
            chunks: Vec::new(),
            output: Vec::new(),
            iter_count: 0,
            item_count: 0,
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

    pub fn run_iteration(&mut self) -> Result<KWayMergeState, ()> {
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
        if self.output.len() >= BUCKET_SIZE {
            self.iter_count += 1;
            self.item_count += self.output.len();
            store_table1_part(&self.output, self.iter_count, Some("_final"));
            self.output.clear();
        }

        // Keeping only unfinished chunks
        self.chunks.retain(|x| !x.is_done());

        if self.chunks.len() == 0 {
            return Ok(KWayMergeState::Done);
        }

        return Ok(KWayMergeState::Success);
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

fn compress<T>(writer: T) -> DeflateEncoder<T>
where
    T: Write,
{
    DeflateEncoder::new(writer, Compression::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_buckets() {}
}
