extern crate fund;

use std::env;
use std::process;

use fund::Config;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::new(&args);

    if let Err(e) = fund::run(config) {
        eprintln!("Application error {}", e);
        process::exit(1);
    }
}
