use chrono::{Duration, NaiveDateTime};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Exp, Normal, Poisson, Triangular, Uniform};
use serde::Serialize;
use std::cmp;

use crate::config;
use crate::constants;
use crate::utils;

#[derive(Debug, PartialEq, Serialize)]
enum FailureReason {
    AccountLocked,
    WrongUsername,
    WrongPassword,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Log {
    datetime: String,
    source_ip: String,
    username: String,
    success: bool,
    failure_reason: Option<FailureReason>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Attack {
    start: String,
    end: String,
    source_ip: String,
}

#[derive(Debug, PartialEq)]
pub struct LoginSimulator {
    config: config::Config,
    userbase: Vec<User>,
    rng: StdRng,
    logs: Vec<Log>,
    attacks: Vec<Attack>,
}
pub struct UserDataset {
    first_names: Vec<String>,
    last_names: Vec<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct User {
    name: String,
    ips: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum DistributionType {
    Triangular { min: f32, mode: f32, max: f32 },
    Uniform { min: f32, max: f32 },
    Normal { mean: f32, std_dev: f32 },
    Exponential { lambda: f32 },
}

impl DistributionType {
    pub fn sample_from_config(
        distribution: &config::Distribution,
        rng: &mut StdRng,
    ) -> Result<f32, String> {
        match distribution.dist_type.to_lowercase().as_str() {
            "triangular" => {
                let dist = Triangular::new(
                    distribution.params["min"],
                    distribution.params["max"],
                    distribution.params["mode"],
                )
                .map_err(|e| e.to_string())?;
                Ok(dist.sample(rng))
            }
            "uniform" => {
                let dist = Uniform::new(distribution.params["min"], distribution.params["max"]);
                Ok(dist.sample(rng))
            }
            "normal" => {
                let dist = Normal::new(distribution.params["mean"], distribution.params["std_dev"])
                    .map_err(|e| e.to_string())?;
                Ok(dist.sample(rng))
            }
            "exponential" => {
                let dist = Exp::new(distribution.params["lambda"]).map_err(|e| e.to_string())?;
                Ok(dist.sample(rng))
            }
            _ => Err(format!("Unknown distribution: {}", distribution.dist_type)),
        }
    }
}

pub trait LoginSimulation {
    fn new(config: config::Config) -> Self;
    fn build_userbase(user_dataset: &UserDataset) -> Vec<User>;
    fn run(self) -> Self;
}

impl LoginSimulation for LoginSimulator {
    fn new(config: config::Config) -> Self {
        let user_dataset = UserDataset {
            first_names: config.first_names.clone(),
            last_names: config.last_names.clone(),
        };
        let seed = config.seed;

        Self {
            config: config,
            rng: StdRng::seed_from_u64(seed),
            userbase: LoginSimulator::build_userbase(&user_dataset),
            logs: vec![],
            attacks: vec![],
        }
    }

    fn build_userbase(user_dataset: &UserDataset) -> Vec<User> {
        let mut user_list: Vec<String> = Vec::new();
        for first in &user_dataset.first_names {
            for last in &user_dataset.last_names {
                user_list.push(format!("{}{}", first, last));
            }
        }
        return LoginSimulator::assign_ip_address(user_list, 3);
    }

    fn run(mut self) -> Self {
        let hours_range = self.config.get_hour_range().unwrap();
        let start =
            NaiveDateTime::parse_from_str(self.config.start_date.as_str(), constants::DATE_FORMAT)
                .unwrap();

        info!("hours range {}", hours_range);
        for offset in 0..hours_range + 1 {
            let mut current = start + Duration::hours(offset);
            if self.rng.gen::<f32>() < self.config.attack_probability {
                let attack_start = current + Duration::minutes(self.rng.gen_range(0..60));

                let subset_size = self.rng.gen_range(1..cmp::max(2, self.userbase.len() / 5));
                let mut random_user_list: Vec<User> = self
                    .userbase
                    .choose_multiple(&mut self.rng, subset_size)
                    .cloned()
                    .collect();

                let (source_ip, end_time) = self.hack(attack_start, &mut random_user_list);
                self.attacks.push(Attack {
                    start: attack_start.to_string(),
                    end: end_time.to_string(),
                    source_ip: source_ip,
                });
            }
            info!("current time {}", current.to_string());
            let (hourly_arrivals, interarrival_times) = self.valid_user_arrivals(current);
            if let Some(random_user) = self.userbase.choose(&mut self.rng) {
                let random_user = random_user.clone();
                for index in 0..hourly_arrivals as usize {
                    current += Duration::minutes(interarrival_times[index] as i64);
                    let username_accuracy = Normal::new(1.01, 0.01)
                        .unwrap()
                        .sample(&mut rand::thread_rng());
                    current = self.attempt_login(
                        &mut current,
                        &random_user.name,
                        username_accuracy,
                    )
                }
            }
            info!("log {:?}", self.logs.last());
            info!("attack {:?}", self.attacks.last());
        }
        self
    }
}

impl LoginSimulator {
    fn assign_ip_address(users: Vec<String>, max_range: u8) -> Vec<User> {
        let mut user_info = Vec::new();
        let mut rng = StdRng::seed_from_u64(13);
        for user in users {
            let ips = (0..rng.gen_range(1..max_range + 1))
                .map(|_| utils::get_random_ip(&mut rng))
                .collect::<Vec<_>>();

            user_info.push(User {
                name: user,
                ips: ips,
            })
        }
        user_info
    }

    fn get_random_user_ip(&mut self, user: &User) -> String {
        user.ips.choose(&mut self.rng).unwrap().to_string()
    }

    fn valid_user_arrivals(&mut self, when: NaiveDateTime) -> (f64, Vec<f64>) {
        let time_period = self.config.get_time_period(when).ok().flatten();
        let poisson_lambda: f64 = if let Some(period) = time_period {
            let sample = DistributionType::sample_from_config(&period.distribution, &mut self.rng);
            sample.unwrap_or(2.0) as f64
        } else {
            2.0 // fallback default
        };
        let hourly_arrivals = Poisson::new(poisson_lambda)
            .unwrap()
            .sample(&mut rand::thread_rng());
        let interarrival_times: Vec<f64> = Exp::new(1.0 / poisson_lambda)
            .unwrap()
            .sample_iter(&mut rand::thread_rng())
            .take(hourly_arrivals as usize)
            .collect();
        return (hourly_arrivals, interarrival_times);
    }

    fn hack(&mut self, when: NaiveDateTime, user_list: &mut Vec<User>) -> (String, NaiveDateTime) {
        // simulate attack from random hacker
        user_list.shuffle(&mut self.rng); // user list is shuffled
        let hacker_ip = utils::get_random_ip(&mut self.rng);
        let mut last_when = when;
        for _user in user_list {
            let new_ip = utils::get_random_ip(&mut self.rng);
            let source_ip = if self.config.vary_ips {
                &new_ip
            } else {
                &hacker_ip
            };
            let username_accuracy = Normal::new(0.35, 0.5)
                .unwrap()
                .sample(&mut rand::thread_rng());
            last_when = self.attempt_login(&mut last_when, source_ip, username_accuracy);
        }
        return (hacker_ip, last_when);
    }

    fn attempt_login(
        &mut self,
        when: &mut NaiveDateTime,
        username: &str,
        username_accuracy: f64,
    ) -> NaiveDateTime {
        let mut login_user = username.to_string();
        if self.rng.gen::<f64>() > username_accuracy {
            // Incorrect username is taken
            login_user = self.distort_username(login_user);
            info!("distorted username - {}", login_user);
        }
        // TODO: implement locked_accounts check and login logic
        *when
    }
    fn distort_username(&mut self, username: String) -> String {
        let mut distorted_username = username.clone(); // avoid mutable borrows
        let index = self.rng.gen_range(0..username.len() - 1);
        if self.rng.gen::<f64>() < 0.5 {
            distorted_username.remove(index);
            distorted_username
        } else {
            let random_char = self.rng.gen_range('a'..='z');
            distorted_username.insert(index, random_char);
            distorted_username
        }
    }
}
