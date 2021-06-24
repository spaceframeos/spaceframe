use structopt::StructOpt;
// use rand::{RngCore, rngs::OsRng};
use spaceframe_pospace::core::PoSpace;

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
        space: usize
    }
}

fn main() {

    let opt = Opts::from_args();

    match opt.cmd {
        Command::Init {space } => {
            // let mut plot_seed = [0u8; 32];
            // OsRng.fill_bytes(&mut plot_seed);
            let plot_seed = b"abcdabcdabcdabcdabcdabcdabcdabcd";
            let mut pos = PoSpace::new(space, plot_seed);
            pos.run_phase_1();
        },
    }
}
