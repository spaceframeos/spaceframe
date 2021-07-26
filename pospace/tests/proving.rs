use log::LevelFilter;
use rand::rngs::OsRng;
use rand::RngCore;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use spaceframe_pospace::core::PoSpace;
use spaceframe_pospace::proofs::Prover;
use tempdir::TempDir;

#[test]
fn test_proving() {
    TermLogger::init(
        LevelFilter::Trace,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    let dir = TempDir::new("spaceframe_proving").unwrap();
    const TEST_K: usize = 14;
    let mut plot_seed = [0u8; 32];
    let mut challenge = [0u8; 32];
    OsRng.fill_bytes(&mut plot_seed);
    OsRng.fill_bytes(&mut challenge);
    let mut pos = PoSpace::new(TEST_K, plot_seed, dir.path());
    pos.run_phase_1();

    let prover = Prover::new(pos);
    prover.retrieve_all_proofs(challenge.as_ref());
}
