use rand::thread_rng;
use rand::Rng;
use spaceframe_pospace::bits::to_bits;
use spaceframe_pospace::sort::sort_table_on_disk;
use spaceframe_pospace::storage::plotentry_size;
use spaceframe_pospace::storage::ENTRIES_PER_CHUNK;
use spaceframe_pospace::storage::{store_table_part, PlotEntry};
use std::fs::File;
use std::io::Read;
use tempdir::TempDir;

fn setup_storage() -> TempDir {
    let dir = TempDir::new("spaceframe_test_data").unwrap();
    let mut rng = thread_rng();
    for i in 0..3 {
        let data = (0..100)
            .map(|x| {
                return PlotEntry {
                    fx: rng.gen_range(0..120),
                    metadata: Some(to_bits(100 * i + x, 12).as_raw_slice().to_vec()),
                    position: None,
                    offset: None,
                };
            })
            .collect::<Vec<PlotEntry>>();
        store_table_part(&data, &dir.path().join(format!("table1_raw_{}", i)));
    }
    dir
}

#[test]
fn test_kway_merge_table1() {
    let dir = setup_storage();
    sort_table_on_disk::<PlotEntry>(1, dir.path(), 10, 12);
    let mut file = File::open(dir.path().join("table1_final")).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let entries = buffer
        .chunks(plotentry_size(1, 12))
        .map(|chunk| {
            return bincode::deserialize(&chunk).unwrap();
        })
        .collect::<Vec<PlotEntry>>();
    let mut last = entries[0].fx;
    assert_eq!(300, entries.len());
    for entry in entries {
        assert!(entry.fx >= last, "Final table not correctly sorted");
        last = entry.fx;
    }
}

#[test]
fn test_kway_merge_table1_big_chunk() {
    let dir = setup_storage();
    sort_table_on_disk::<PlotEntry>(1, dir.path(), *ENTRIES_PER_CHUNK, 12);
    let mut file = File::open(dir.path().join("table1_final")).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let entries = buffer
        .chunks(plotentry_size(1, 12))
        .map(|chunk| {
            return bincode::deserialize(&chunk).unwrap();
        })
        .collect::<Vec<PlotEntry>>();
    let mut last = entries[0].fx;
    assert_eq!(300, entries.len());
    for entry in entries {
        assert!(entry.fx >= last, "Final table not correctly sorted");
        last = entry.fx;
    }
}
