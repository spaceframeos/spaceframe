use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use spaceframe_pospace::core::PoSpace;
use spaceframe_pospace::proofs::Prover;
use spaceframe_pospace::verifier::Verifier;
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
    let plot_seed = *b"aaaabbbbccccddddaaaabbbbccccdddd";
    let challenge = [
        180, 152, 16, 199, 88, 233, 76, 61, 6, 3, 95, 26, 98, 214, 224, 127, 19, 87, 188, 143, 134,
        79, 228, 168, 126, 117, 83, 103, 121, 41, 79, 94,
    ];
    let pos = PoSpace::new(TEST_K, plot_seed, dir.path()).unwrap();
    pos.run_phase_1().unwrap();

    let prover = Prover::new(pos);
    let proofs = prover.retrieve_all_proofs(challenge.as_ref()).unwrap();
    assert_eq!(proofs.len(), 1);

    let verifier = Verifier::new();
    assert!(verifier.verify_proof(&proofs[0]).is_ok(), "Invalid proof");
}
