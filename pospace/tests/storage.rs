use glob::glob;
use rand::thread_rng;
use rand::Rng;
use spaceframe_pospace::storage::sort_table;
use spaceframe_pospace::storage::TABLE1_SERIALIZED_ENTRY_SIZE;
use spaceframe_pospace::storage::{store_table1_part, Table1Entry};
use std::fs::create_dir_all;
use std::fs::remove_dir_all;
use std::fs::File;
use std::io::Read;

fn setup_storage() {
    let mut rng = thread_rng();
    remove_dir_all("test_data").ok();
    create_dir_all("test_data").ok();
    for i in 0..3 {
        let data = (0..100)
            .map(|x| {
                return Table1Entry {
                    x: 100 * i + x,
                    y: rng.gen_range(0..120),
                };
            })
            .collect::<Vec<Table1Entry>>();
        store_table1_part(&data, "test_data", i as usize, None);
    }
}

#[test]
fn test_kway_merge() {
    setup_storage();
    sort_table("test_data", "test_data/table1_*", 10);
    let entries = glob("test_data/table1_*_final")
        .unwrap()
        .filter_map(Result::ok)
        .flat_map(|f| {
            let mut file = File::open(f).unwrap();
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).unwrap();
            buffer
                .chunks(*TABLE1_SERIALIZED_ENTRY_SIZE)
                .map(|chunk| {
                    return bincode::deserialize(&chunk).unwrap();
                })
                .collect::<Vec<Table1Entry>>()
        })
        .collect::<Vec<Table1Entry>>();
    let mut last = entries[0].y;
    for entry in entries {
        assert!(entry.y >= last, "Final table not correctly sorted");
        last = entry.y;
    }
}
