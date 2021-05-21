use clap::{App, Arg, SubCommand, crate_authors, crate_name, crate_version};

use spaceframe_pospace::init_pos;

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
            init_pos(k);
        }
    }
}
