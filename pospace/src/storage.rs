use std::{fs::File, io::Write, path::Path};

use crate::bits::BitsWrapper;
use flate2::{write::DeflateEncoder, Compression};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Table1Entry {
    pub x: u64,
    pub y: u64,
}

pub struct PlotEntry {
    pub y: BitsWrapper,
    pub pos: u64,
    pub offset: u64,
}

pub fn store_table1_part(buffer: &[Table1Entry], index: usize) {
    let new_file = File::create(Path::new("data").join(format!("table1_{}", index))).unwrap();
    let bin_data = bincode::serialize(&buffer).unwrap();
    let mut new_file = compress(new_file);
    new_file.write_all(&bin_data).unwrap();
}

fn compress<T>(writer: T) -> DeflateEncoder<T>
where
    T: Write,
{
    DeflateEncoder::new(writer, Compression::default())
}
