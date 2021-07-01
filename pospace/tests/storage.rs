use rand::thread_rng;
use rand::Rng;
use spaceframe_pospace::storage::sort_table_on_disk;
use spaceframe_pospace::storage::TABLE1_SERIALIZED_ENTRY_SIZE;
use spaceframe_pospace::storage::{store_table_part, Table1Entry};
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn setup_storage() {
    let mut rng = thread_rng();
    for i in 0..3 {
        let data = (0..100)
            .map(|x| {
                return Table1Entry {
                    x: 100 * i + x,
                    y: rng.gen_range(0..120),
                };
            })
            .collect::<Vec<Table1Entry>>();
        store_table_part(
            &data,
            &Path::new("test_data").join(format!("table1_raw_{}", i)),
        );
    }
}

#[test]
fn test_kway_merge_table1() {
    setup_storage();
    sort_table_on_disk::<Table1Entry>(1, "test_data", "test_data/table1_raw_*", 10);
    let mut file = File::open("test_data/table1_final").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let entries = buffer
        .chunks(*TABLE1_SERIALIZED_ENTRY_SIZE)
        .map(|chunk| {
            return bincode::deserialize(&chunk).unwrap();
        })
        .collect::<Vec<Table1Entry>>();
    let mut last = entries[0].y;
    assert_eq!(300, entries.len());
    for entry in entries {
        assert!(entry.y >= last, "Final table not correctly sorted");
        last = entry.y;
    }
}

#[test]
fn test_kway_merge_table1_big_chunk() {
    setup_storage();
    sort_table_on_disk::<Table1Entry>(1, "test_data", "test_data/table1_*", 100);
    let mut file = File::open("test_data/table1_final").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let entries = buffer
        .chunks(*TABLE1_SERIALIZED_ENTRY_SIZE)
        .map(|chunk| {
            return bincode::deserialize(&chunk).unwrap();
        })
        .collect::<Vec<Table1Entry>>();
    let mut last = entries[0].y;
    assert_eq!(300, entries.len());
    for entry in entries {
        assert!(entry.y >= last, "Final table not correctly sorted");
        last = entry.y;
    }
}
