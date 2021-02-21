#[macro_use]
extern crate clap;
use clap::App;

#[macro_use]
extern crate log;
extern crate simplelog;

use std::path::Path;
use std::fs;

pub mod feeder;

fn main() {
    //parse cmdline
    let yaml = load_yaml!("cmdline.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    let token_file = String::from(matches.value_of("token").unwrap());
    let vss_host   = String::from(matches.value_of("host").unwrap());

    let debug = matches.is_present("debug");

    let mut level = simplelog::LevelFilter::Info;
    if debug {
        level = simplelog::LevelFilter::Debug;
    }

    //init logging framework
    simplelog::CombinedLogger::init(vec![
        simplelog::TermLogger::new(level, simplelog::Config::default(), simplelog::TerminalMode::Mixed)
    ])
    .unwrap();

    info!("Will use JWT token in......: {}", token_file);
    info!("Will connect to VSS server.: {}", vss_host);

    if  !Path::new(&token_file).exists() || !Path::new(&token_file).is_file()  {
        error!("JWT token file {} does not exist or is not a file. Exiting.", token_file);
        return;
    }

    let token = fs::read_to_string(token_file)
        .expect("Error reading the token file"); 

    feeder::start_feeder(&vss_host, &token);
}
