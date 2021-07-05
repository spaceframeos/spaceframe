use rand::thread_rng;
use rand::Rng;
use spaceframe_pospace::storage::sort_table_on_disk;
use spaceframe_pospace::storage::ENTRIES_PER_CHUNK;
use spaceframe_pospace::storage::TABLE1_SERIALIZED_ENTRY_SIZE;
use spaceframe_pospace::storage::{store_table_part, Table1Entry};
use std::fs::File;
use std::io::Read;
use tempdir::TempDir;

fn setup_storage() -> TempDir {
    let dir = TempDir::new("spaceframe_test_data").unwrap();
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
        store_table_part(&data, &dir.path().join(format!("table1_raw_{}", i)));
    }
    dir
}

#[test]
fn test_kway_merge_table1() {
    let dir = setup_storage();
    sort_table_on_disk::<Table1Entry>(1, dir.path(), 10);
    let mut file = File::open(dir.path().join("table1_final")).unwrap();
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
    let dir = setup_storage();
    sort_table_on_disk::<Table1Entry>(1, dir.path(), ENTRIES_PER_CHUNK);
    let mut file = File::open(dir.path().join("table1_final")).unwrap();
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
