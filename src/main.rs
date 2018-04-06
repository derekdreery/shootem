//! Demonstrates how to use the fly camera

extern crate amethyst;
extern crate pretty_env_logger;
extern crate genmesh;

mod game_state;

use game_state::run;
use std::env;

fn main() {
    // Turn logging on by default
    match env::var("RUST_LOG") {
        Err(env::VarError::NotPresent) => {
            env::set_var("RUST_LOG", "info,gfx_device_gl=warn");
        }
        _ => (),
    }

    // use pretty logging
    pretty_env_logger::init();

    // Do some basic error reporting
    if let Err(error) = run() {
        eprintln!("Could not run the example!");
        eprintln!("{}", error);
        ::std::process::exit(1);
    }
}
