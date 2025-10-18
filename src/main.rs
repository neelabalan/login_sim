use std::env;
use std::fs;

mod config;
mod constants;
mod simulator;
mod utils;

#[macro_use]
extern crate log;

use config::Config;
use env_logger::Env;

use crate::simulator::LoginSimulation;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Usage: {} <config.json>", args[0]);
    }
    let config_file = &args[1];
    let config_content = fs::read_to_string(config_file).expect("Unable to read config file");
    let config: Config =
        serde_json::from_str(&config_content).expect("Unable to parse config file");

    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let simulator = simulator::LoginSimulator::new(config);
    let simulator = simulator.run();

    simulator.dump_userbase()?;
    simulator.dump_logs()?;
    simulator.dump_attacks()?;

    Ok(())
}
