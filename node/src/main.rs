use base64::encode;
use clap::{App, Arg, SubCommand, crate_authors, crate_name, crate_version};

use rand::{RngCore, rngs::OsRng};
use spaceframe_pospace::{core::PoSpace};

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about("Node manager")
        .subcommand(SubCommand::with_name("init")
            .about("Initilize proof-of-space")
            .arg(Arg::with_name("space")
                .short("k")
                .required(true)
                .takes_value(true)
                .help("Space parameter [10-30]")
            )
        )
        .subcommand(SubCommand::with_name("start")
            .about("Start the node")
            .arg(Arg::with_name("network")
                .short("n")
                .takes_value(true)
                .possible_values(&["dev", "test", "main"])
                .default_value("dev")
                .help("Which network to connect the node")
            )
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("init") {
        if let Ok(k) = matches.value_of("space").unwrap().parse::<usize>() {
            let mut plot_seed = [0u8; 24];
            OsRng.fill_bytes(&mut plot_seed);
            let pos = PoSpace::new(k, &encode(plot_seed));
            pos.run_phase_1();
        }
    }
}
