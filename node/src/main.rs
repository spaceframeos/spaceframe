use clap::App;

use spaceframe_pospace::init_pos;

fn main() {
    let _matches = App::new("Spaceframe Node")
        .version("0.0.1")
        .author("Gil Balsiger <gil.balsiger@gmail.com>")
        .about("Spaceframe binary to create a node")
        .get_matches();

    init_pos();
}
