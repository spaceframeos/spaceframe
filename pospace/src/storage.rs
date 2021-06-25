use glob::glob;
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use crate::bits::BitsWrapper;
use flate2::{write::DeflateEncoder, Compression};
use serde::{Deserialize, Serialize, __private::de::InternallyTaggedUnitVisitor, ser};

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
    for entry in glob("data/table1_*").unwrap() {
        match entry {
            Ok(path) => {
                if path.is_file() {
                    let mut buffer = Vec::new();
                    let mut file = File::open(&path).unwrap();
                    let amount = file.read_to_end(&mut buffer).unwrap();
                    let serialized_size =
                        bincode::serialized_size(&Table1Entry { x: 12345, y: 12345 }).unwrap();
                    println!(
                        "Amount: {} Kb, ser size: {} - {}",
                        amount / 1024,
                        serialized_size,
                        amount as f64 / serialized_size as f64
                    );
                    let mut entries = buffer
                        .chunks(serialized_size as usize)
                        .map(|chunk| {
                            return bincode::deserialize(&chunk).unwrap();
                        })
                        .collect::<Vec<Table1Entry>>();

                    entries.sort();
                    let mut sorted_file = File::create(Path::new("data").join(format!(
                        "{}_sorted",
                        String::from(path.file_name().unwrap().to_str().unwrap())
                    )))
                    .unwrap();

                    let bin_data = entries
                        .iter()
                        .flat_map(|x| bincode::serialize(x).unwrap())
                        .collect::<Vec<u8>>();

                    sorted_file
                        .write_all(&bincode::serialize(&bin_data).unwrap())
                        .unwrap();
                }
            }
            Err(_) => todo!(),
        }
    }

    // K-Way Merge sort
}

fn compress<T>(writer: T) -> DeflateEncoder<T>
where
    T: Write,
{
    DeflateEncoder::new(writer, Compression::default())
}
