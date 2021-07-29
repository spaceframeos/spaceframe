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
            let mut pos = PoSpace::new(space, plot_seed, "data".as_ref())
                .context("Failed to create proof of space instance")?;
            pos.run_phase_1()
                .context("Failed to run phase 1 of plotting")
        }
        Command::Prove { space } => {
            let mut challenge = [0u8; 32];
            OsRng.fill_bytes(&mut challenge);
            let pos = PoSpace::new(space, *b"aaaabbbbccccddddaaaabbbbccccdddd", "data".as_ref())
                .context("Failed to create proof of space instance")?;
            let prover = Prover::new(pos);
            // prover.get_quality_string(challenge.as_ref());
            let proofs = prover
                .retrieve_all_proofs(challenge.as_ref())
                .context("Cannot retrieve all proofs for challange")?;

            let verifier = Verifier::new();

            for proof in proofs {
                if verifier.verify_proof(&proof) {
                    info!("Valid proof");
                } else {
                    warn!("Invalid proof");
                }
            }

            Ok(())
        }
    }
}
