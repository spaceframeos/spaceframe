use log::info;
use rand::{rngs::OsRng, RngCore};
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
use spaceframe_pospace::core::PoSpace;
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
}

fn main() {
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
            // let plot_seed = b"abcdabcdabcdabcdabcdabcdabcdabcd";
            let mut pos = PoSpace::new(space, &plot_seed);
            pos.run_phase_1();
        }
    }
}
