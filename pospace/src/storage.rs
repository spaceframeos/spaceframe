use crate::core::collation_size_bits;
use crate::error::StorageError;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
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

pub fn store_table_part<T>(buffer: &[T], path: &Path)
where
    T: Serialize + Debug,
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
    T: Serialize + Debug,
{
    buffer
        .iter()
        .flat_map(|entry| {
            let value = bincode::serialize(entry).unwrap();
            // debug!("Size of {:?} is {}", entry, value.len());
            return value;
        })
        .collect::<Vec<u8>>()
}

pub fn deserialize(buffer: &[u8], entry_size: usize) -> Result<Vec<PlotEntry>> {
    let result = buffer
        .chunks(entry_size)
        .map(|chunk| Ok(bincode::deserialize(&chunk).or(Err(StorageError::DeserializationError))?))
        .collect::<Result<Vec<PlotEntry>>>()?;
    Ok(result)
}

// Size in bytes
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
        store_table_part(&test_data, &path);

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
