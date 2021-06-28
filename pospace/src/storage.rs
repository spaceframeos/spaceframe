use glob::glob;
use std::{
    fs::File,
    io::{Read, Write},
    os::unix::prelude::MetadataExt,
    path::Path,
};

use crate::bits::BitsWrapper;
use flate2::{write::DeflateEncoder, Compression};
use serde::{Deserialize, Serialize};

pub const BUCKET_SIZE: usize = 20_000;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
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

pub fn store_table1_part(buffer: &[Table1Entry], index: usize) {
    let mut new_file = File::create(Path::new("data").join(format!("table1_{}", index))).unwrap();
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
    let serialized_size =
        bincode::serialized_size(&Table1Entry { x: 12345, y: 12345 }).unwrap() as usize;

    for entry in glob("data/table1_*").unwrap() {
        match entry {
            Ok(path) => {
                if path.is_file() {
                    let mut buffer = Vec::new();
                    let mut file = File::open(&path).unwrap();
                    file.read_to_end(&mut buffer).unwrap();
                    let mut entries = buffer
                        .chunks(serialized_size as usize)
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
    println!("K-Way merging for table 1 ...");

    // Load BUCKET_SIZE / (chunks_count - 1) buckets into RAM
    if chunks_count > 1 {
        let merge_chunck_size = BUCKET_SIZE / (chunks_count - 1);
        let mut state = KWayMergeState {
            chunks: Vec::new(),
            output: Vec::new(),
        };

        for path in parts {
            let mut file = File::open(path).unwrap();
            let mut buffer = vec![0u8; serialized_size * merge_chunck_size];
            let amount = file.read(&mut buffer).unwrap();
            println!("Red {} bytes", amount);
            let entries = buffer
                .chunks(serialized_size as usize)
                .filter_map(|chunk| {
                    if chunk.iter().all(|c| c == &0u8) {
                        return None;
                    } else {
                        return Some(bincode::deserialize(&chunk).unwrap());
                    }
                })
                .collect::<Vec<Table1Entry>>();
            let total_size = file.metadata().unwrap().len() as usize / serialized_size;
            state.chunks.push(MergeChunk {
                remaining: total_size - entries.len(),
                content: entries,
                indice: 0,
                total_size,
            });
        }

        println!(
            "{:?}",
            state
                .chunks
                .iter()
                .map(|p| p.content.len())
                .collect::<Vec<usize>>()
        );

        println!(
            "{:?}",
            state
                .chunks
                .iter()
                .map(|p| p.remaining)
                .collect::<Vec<usize>>()
        )

        // println!("State: {:?}", state);
    }
}

#[derive(Debug)]
struct KWayMergeState {
    chunks: Vec<MergeChunk>,
    output: Vec<Table1Entry>,
}

#[derive(Debug)]
struct MergeChunk {
    content: Vec<Table1Entry>,
    indice: usize,
    total_size: usize,
    remaining: usize,
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
