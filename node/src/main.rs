use anyhow::Context;
use anyhow::Result;
use log::*;
use rand::{rngs::OsRng, RngCore};
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
use spaceframe_pospace::core::PoSpace;
use spaceframe_pospace::proofs::Prover;
use spaceframe_pospace::verifier::Verifier;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "spaceframe-node", author = "Gil Balsiger")]
struct Opts {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    /// Initialize the proofs of space
    Init {
        #[structopt(short = "k")]
        space: usize,
    },

    Prove {
        #[structopt(short = "k")]
        space: usize,
    },
}

const fn get_challenge(k: usize) -> [u8; 32] {
    match k {
        17 => [
            95, 23, 106, 107, 81, 99, 43, 112, 157, 49, 246, 48, 199, 114, 163, 190, 160, 165, 251,
            13, 92, 80, 240, 210, 241, 247, 44, 74, 94, 126, 245, 226,
        ],
        19 => [
            140, 31, 177, 106, 121, 35, 250, 68, 109, 103, 251, 149, 126, 201, 224, 230, 37, 74,
            247, 24, 146, 131, 28, 74, 17, 105, 126, 93, 105, 34, 222, 152,
        ],
        20 => [
            8, 69, 62, 233, 63, 175, 0, 92, 104, 211, 47, 131, 61, 52, 7, 0, 19, 150, 63, 103, 88,
            212, 133, 181, 140, 197, 12, 27, 33, 249, 33, 196,
        ],
        21 => [
            154, 156, 38, 140, 105, 6, 177, 113, 168, 152, 154, 83, 173, 244, 200, 201, 218, 49,
            102, 110, 98, 200, 99, 103, 187, 151, 182, 107, 149, 19, 244, 32,
        ],
        22 => [
            95, 173, 51, 119, 88, 121, 172, 0, 127, 130, 7, 43, 153, 16, 149, 83, 31, 188, 77, 86,
            226, 139, 33, 67, 232, 168, 112, 191, 24, 57, 130, 138,
        ],
        23 => [
            140, 68, 99, 54, 232, 127, 7, 76, 16, 142, 107, 249, 168, 126, 107, 139, 8, 163, 255,
            57, 54, 221, 139, 159, 175, 120, 246, 228, 68, 141, 187, 12,
        ],
        _ => [0u8; 32],
    }
}

fn main() -> Result<()> {
    let opt = Opts::from_args();

    TermLogger::init(
        LevelFilter::Trace,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    match opt.cmd {
        Command::Init { space } => {
            let mut plot_seed = [0u8; 32];
            OsRng.fill_bytes(&mut plot_seed);
            info!("Plot seed generated");
            let pos = PoSpace::new(space, *b"aaaabbbbccccddddaaaabbbbccccdddd", "data".as_ref())
                .context("Failed to create proof of space instance")?;
            pos.run_phase_1()
                .context("Failed to run phase 1 of plotting")
        }
        Command::Prove { space } => {
            let challenge = get_challenge(space);
            // OsRng.fill_bytes(&mut challenge);
            let pos = PoSpace::new(space, *b"aaaabbbbccccddddaaaabbbbccccdddd", "data".as_ref())
                .context("Failed to create proof of space instance")?;
            let prover = Prover::new(pos);
            // prover.get_quality_string(challenge.as_ref());
            prover
                .retrieve_all_proofs(challenge.as_ref())
                .context("Cannot retrieve all proofs for challange")?;

            // let verifier = Verifier::new();

            // for proof in proofs {
            //     if verifier.verify_proof(&proof) {
            //         info!("Valid proof");
            //     } else {
            //         warn!("Invalid proof");
            //     }
            // }

            Ok(())
        }
    }
}
