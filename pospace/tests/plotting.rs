use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use spaceframe_pospace::core::PoSpace;
use tempdir::TempDir;

#[test]
fn test_plotting() {
    TermLogger::init(
        LevelFilter::Trace,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    let dir = TempDir::new("spaceframe_plotting").unwrap();
    const TEST_K: usize = 14;
    let plot_seed = *b"aaaabbbbccccddddaaaabbbbccccdddd";
    let pos = PoSpace::new(TEST_K, plot_seed, dir.path()).unwrap();
    pos.run_phase_1().unwrap();
}
