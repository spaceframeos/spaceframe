use crate::core::collation_size_bits;
use crate::error::StorageError;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::io::Read;
use std::{fs::File, io::Write, path::Path};
use sysinfo::SystemExt;

lazy_static! {
    pub static ref ENTRIES_PER_CHUNK: usize = {
        let mut system = sysinfo::System::new_all();
        system.refresh_all();
        let power: u64 = (system.total_memory() as f64).log(2f64) as u64;
        1 << power
    };
}

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
    pub metadata: Option<Vec<u8>>,
    pub position: Option<u64>,
    pub offset: Option<u64>,
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

pub fn store_table_part(buffer: &[PlotEntry], path: &Path) -> Result<()> {
    let mut new_file = File::create(path).context(format!("Could not create file {:?}", path))?;
    let bin_data = serialize(buffer).context("Could not serialize table part")?;
    new_file
        .write_all(&bin_data)
        .context("Could not write table part to disk")?;
    Ok(())
}

pub fn store_raw_table_part(
    table_index: usize,
    part_index: usize,
    buffer: &[PlotEntry],
    path: &Path,
) -> Result<()> {
    store_table_part(
        buffer,
        &path.join(format!(
            table_raw_filename_format!(),
            table_index, part_index
        )),
    )
}

fn ser(entry: &PlotEntry) -> Result<Vec<u8>> {
    Ok(bincode::serialize(entry).or(Err(StorageError::SerializationError))?)
}

pub fn serialize(buffer: &[PlotEntry]) -> Result<Vec<u8>> {
    let res = buffer.iter().map(ser).collect::<Result<Vec<Vec<u8>>>>()?;
    Ok(res.iter().flatten().cloned().collect::<Vec<u8>>())
}

fn deser(chunk: &[u8]) -> Result<PlotEntry> {
    Ok(bincode::deserialize(&chunk).or(Err(StorageError::DeserializationError))?)
}

pub fn deserialize(buffer: &[u8], entry_size: usize) -> Result<Vec<PlotEntry>> {
    buffer
        .chunks(entry_size)
        .map(deser)
        .collect::<Result<Vec<PlotEntry>>>()
}

/// Size in bytes
pub fn plotentry_size(table_index: usize, k: usize) -> usize {
    let metadata_size = (collation_size_bits(table_index + 1, k) as f64 / 8 as f64).ceil() as usize;
    let position_size = 8;
    let offset_size = 8;
    let fx_size = 8;

    return 11
        + match table_index {
            1 => fx_size + metadata_size,
            7 => fx_size + offset_size + position_size,
            _ => fx_size + metadata_size + offset_size + position_size,
        };
}

pub struct ChunkReader {
    pub remaining_size: usize,
    pub entry_size: usize,
    pub file: File,
}

impl ChunkReader {
    pub fn new(path: &Path, table_index: usize, k: usize) -> Result<Self> {
        let file =
            File::open(path.join(format!(table_final_filename_format!(), table_index))).context(
                format!("Cannot open final plot file for table {}", table_index),
            )?;
        let file_size = file.metadata()?.len() as usize;
        let entry_size = plotentry_size(table_index, k);
        let remaining_size = file_size;

        if file_size % entry_size != 0 {
            return Err(StorageError::InvalidFileContent.into());
        }

        Ok(ChunkReader {
            file,
            entry_size,
            remaining_size,
        })
    }

    pub fn read_chunk(&mut self) -> Result<Vec<PlotEntry>> {
        if self.remaining_size == 0 {
            return Err(StorageError::EndOfFile.into());
        }
        let mut buffer;
        if self.remaining_size > *ENTRIES_PER_CHUNK * self.entry_size {
            buffer = vec![0; *ENTRIES_PER_CHUNK * self.entry_size];
            self.file.read_exact(&mut buffer)?;
            self.remaining_size -= *ENTRIES_PER_CHUNK * self.entry_size;
        } else {
            buffer = Vec::new();
            let amount = self.file.read_to_end(&mut buffer)?;
            self.remaining_size -= amount;
        }
        let entries: Vec<PlotEntry> = deserialize(&buffer, self.entry_size)?;
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {

    use tempdir::TempDir;

    use super::*;
    use crate::bits::to_bits;
    use std::io::Read;

    #[test]
    fn test_store_table_part_table1() -> Result<()> {
        let test_k = 12;
        let dir = TempDir::new("spaceframe_test_data").unwrap();
        let test_data = vec![
            PlotEntry {
                fx: 2,
                metadata: Some(to_bits(3, test_k).as_raw_slice().to_vec()),
                position: None,
                offset: None,
            },
            PlotEntry {
                fx: 6,
                metadata: Some(to_bits(1, test_k).as_raw_slice().to_vec()),
                position: None,
                offset: None,
            },
        ];
        let path = dir.path().join("store_table_1");
        store_table_part(&test_data, &path).unwrap();

        let mut verify_buffer = Vec::new();
        File::open(&path)
            .unwrap()
            .read_to_end(&mut verify_buffer)
            .unwrap();
        let verify_data: Vec<PlotEntry> = deserialize(&verify_buffer, plotentry_size(1, test_k))?;

        assert_eq!(test_data, verify_data);
        Ok(())
    }
}
