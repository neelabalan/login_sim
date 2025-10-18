use chrono::NaiveDateTime;
use std::env;
use std::fs;

mod config;
mod simulator;
mod utils;
mod constants;

#[macro_use]
extern crate log;

use config::Config;
use env_logger::Env;

use crate::simulator::LoginSimulation;

macro_rules! vec_of_strings {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Usage: {} <config.json>", args[0]);
    }
    let config_file = &args[1];
    let config_content = fs::read_to_string(config_file).expect("Unable to read config file");
    let config: Config =
        serde_json::from_str(&config_content).expect("Unable to parse config file");

    let first_names = config.first_names;
    let last_names = config.last_names;

    let start = NaiveDateTime::parse_from_str(&config.start_date, "%Y-%m-%d %H:%M:%S").unwrap();
    let end = NaiveDateTime::parse_from_str(&config.end_date, "%Y-%m-%d %H:%M:%S").unwrap();
    let days = (end - start).num_days() as u64;

    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let mut user_list: Vec<String> = Vec::new();
    for first in &first_names {
        for last in &last_names {
            user_list.push(format!("{}{}", first, last));
        }
    }
    let roles = vec_of_strings!["admin", "dba", "master"];

    user_list.extend(roles);

    let mut simulator = simulator::LoginSimulator::new(config);
    let _ = simulator.run();

    // FIX

    utils::dump_json(
        &simulator.userbase,
        config.output.get("userbase").unwrap().as_str(),
    )?;
    let _ = utils::dump_json(&simulator.logs, config.output.get("logs").unwrap().as_str());
    let _ = utils::dump_json(
        &simulator.attacks,
        config.output.get("attacks").unwrap().as_str(),
    );

    Ok(())
}
