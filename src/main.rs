
use clap::Parser;
use itertools::iproduct;

mod utils;
mod simulator;

#[macro_use]
extern crate log;


use rand::seq::SliceRandom;
use std::collections::HashMap;
use rand::thread_rng;
use env_logger::Env;

macro_rules! vec_of_strings {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t=30, help="Simualtion ranging from the start date")]
    days: u64,

    #[arg(long, default_value_t=format!("2022-01-01 00:00:00"), help="Expected format %Y-%m-%d %H:%M:%S")]
    start_date: String,

    #[arg(long, default_value_t=13)]
    seed: u64,

    // filepath
    #[arg(long, default_value_t=format!("logs/ips.json"))]
    ip: String,

    // filepath
    #[arg(long, default_value_t=format!("logs/log.csv"))]
    log: String,

    // file path
    #[arg(long, default_value_t=format!("logs/attack.csv"))]
    hacklog: String,
}


fn get_random_user_ip(userbase: &HashMap<String, Vec<String>>, user_name: &String) -> String {
	let mut rng = thread_rng();
	return userbase
		.get(user_name)
		.unwrap()
		.choose(&mut rng)
		.unwrap()
		.to_string();
}
fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let first_names = utils::load_from_file("data/first_names.txt");
    let last_names = utils::load_from_file("data/last_names.txt");
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);


    let mut user_list : Vec<String> = iproduct!(first_names, last_names)
        .map(|(first_name, last_name)| format!("{}{}", first_name, last_name))
        .collect();
    let roles = vec_of_strings!["admin", "dba", "master"];

    user_list.extend(roles);

    let userbase = utils::assign_ip_address(user_list, 3);
	// for _ in 0..100 {
	// 	info!("ip for AlexHarvey - {}", get_random_user_ip(&userbase, &"AlexHanson".to_string()));
	// }
    let mut simulator: simulator::Simulator = simulator::Simulator::new(
        args.start_date.as_str(),
        args.days,
        vec![0.25, 0.45],
        vec![0.87, 0.93, 0.95],
        args.seed,
        &userbase,
    );
    simulator.simulate(0.1, 0.2, false);

	utils::dump_json(&simulator.userbase, &args.ip)?;
	utils::dump_csv(&simulator.logs, &args.log);
	utils::dump_csv(&simulator.attacks, &args.hacklog);
    Ok(())
}
