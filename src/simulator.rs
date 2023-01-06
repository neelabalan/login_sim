use chrono::format::ParseError;
use chrono::{Datelike, Days, Timelike, Weekday};
use chrono::{Duration, NaiveDateTime};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Exp, Normal, Poisson, Triangular, Uniform};
use serde::Serialize;
use std::cmp;
use std::collections::HashMap;

use crate::utils;

const ATTEMPTS_BEFORE_LOCKOUT: usize = 3;

#[derive(Debug, PartialEq, Serialize)]
enum FailureReason {
    ErrorAccountLocked,
    ErrorWrongUsername,
    ErrorWrongPassword,
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
pub struct Simulator<'a> {
    start_date: &'a str,
    days: u64,
    attacker_success_probs: Vec<f64>,
    valid_user_success_probs: Vec<f64>,
    seed: u64,
    locked_accounts: Vec<String>,
    rng: StdRng,
    pub userbase: &'a HashMap<String, Vec<String>>,
    pub logs: Vec<Log>,
    pub attacks: Vec<Attack>,
}

impl<'a> Simulator<'a> {
    pub fn new(
        start_date: &'a str,
        days: u64,
        attacker_success_probs: Vec<f64>,
        valid_user_success_probs: Vec<f64>,
        seed: u64,
        userbase: &'a HashMap<String, Vec<String>>,
    ) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        Self {
            start_date,
            days,
            attacker_success_probs,
            valid_user_success_probs,
            seed,
            userbase,
            logs: vec![],
            attacks: vec![],
            locked_accounts: vec![],
            rng,
        }
    }
    pub fn simulate(&mut self, attack_prob: f64, try_all_users_prob: f64, vary_ips: bool) {
        let hours_range = &self.get_hour_range().unwrap();
        let start = NaiveDateTime::parse_from_str(&self.start_date, "%Y-%m-%d %H:%M:%S").unwrap();
        let mut user_list: Vec<String> = self.userbase.keys().cloned().collect();

        // info!("User list {:?}", user_list);
        info!("Hours range {}", hours_range);
        for offset in 0..hours_range + 1 {
            let mut current = start + Duration::hours(offset);
            if self.rng.gen::<f64>() < attack_prob {
                let attack_start = current + Duration::minutes(self.rng.gen_range(0..60));

                let mut random_user_list: Vec<String> =
                    if self.rng.gen::<f64>() < try_all_users_prob {
                        user_list.clone() // unaffected
                    } else {
                        let mut temp_rng = rand::thread_rng();
                        let temp_user_list = user_list.clone();
                        temp_user_list
                            .choose_multiple(&mut self.rng, temp_rng.gen_range(0..user_list.len()))
                            .collect::<Vec<&String>>()
                            .into_iter()
                            .map(|s| s.to_owned())
                            .collect()
                    };

                let (source_ip, end_time) =
                    self.hack(attack_start, &mut random_user_list, vary_ips);
                self.attacks.push(Attack {
                    start: attack_start.to_string(),
                    end: end_time.to_string(),
                    source_ip: source_ip,
                });
            }
            info!("Current time {}", current.to_string());
            let (hourly_arrivals, interarrival_times) = Simulator::valid_user_arrivals(current);
            let random_user = user_list.choose(&mut self.rng).unwrap();
            for index in 0..hourly_arrivals as usize {
                current = current + Duration::minutes(interarrival_times[index] as i64);
                current = self.valid_user_attempts_login(&mut current, random_user.clone());
            }
            info!("Log {:?}", self.logs.last());
            info!("Attack {:?}", self.attacks.last());
        }
    }
    fn get_random_user_ip(&mut self, user_name: &String) -> String {
        return self
            .userbase
            .get(user_name)
            .unwrap()
            .choose(&mut self.rng)
            .unwrap()
            .to_string();
    }
    fn valid_user_arrivals(when: NaiveDateTime) -> (f64, Vec<f64>) {
        let is_weekday = ![Weekday::Sat, Weekday::Sun].contains(&when.weekday());
        let late_night = when.hour() < 5 || when.hour() >= 11;
        let work_time = is_weekday && (when.hour() >= 9 || when.hour() <= 17);
        let mut poisson_lambda: f64 = 0.0;
        if work_time {
            let tri_distr = Triangular::new(1.5, 5.0, 2.75).unwrap();
            poisson_lambda = tri_distr.sample(&mut rand::thread_rng());
        } else if late_night {
            let uniform_distr = Uniform::new(0.0, 5.0);
            poisson_lambda = uniform_distr.sample(&mut rand::thread_rng());
        } else {
            let uniform_distr = Uniform::new(1.5, 4.25);
            poisson_lambda = uniform_distr.sample(&mut rand::thread_rng());
        }
        let poisson_distr = Poisson::new(poisson_lambda).unwrap();
        let hourly_arrivals = poisson_distr.sample(&mut rand::thread_rng());
        let exp_distr = Exp::new(1.0 / poisson_lambda).unwrap();
        let interarrival_times: Vec<f64> = exp_distr
            .sample_iter(&mut rand::thread_rng())
            .take(hourly_arrivals as usize)
            .collect();
        return (hourly_arrivals, interarrival_times);
    }
    fn valid_user_attempts_login(
        &mut self,
        current: &mut NaiveDateTime,
        random_user: String,
    ) -> NaiveDateTime {
        let source_ip = self.get_random_user_ip(&random_user);
		debug!("{}-{}", random_user, source_ip);
        let normal_distr = Normal::new(1.01, 0.01).unwrap();
        return self.attempt_login(
            current,
            &source_ip,
            &random_user,
            normal_distr.sample(&mut rand::thread_rng()),
            self.valid_user_success_probs.clone(),
        );
    }
    fn get_hour_range(&self) -> Result<i64, ParseError> {
        let start = NaiveDateTime::parse_from_str(self.start_date, "%Y-%m-%d %H:%M:%S")?;
        let end = start.checked_add_days(Days::new(self.days)).unwrap();

        Ok((end - start).num_hours())
    }
    fn hack(
        &mut self,
        when: NaiveDateTime,
        user_list: &mut Vec<String>,
        vary_ips: bool,
    ) -> (String, NaiveDateTime) {
        // simulate attack from random hacker
        user_list.shuffle(&mut self.rng); // user list is shuffled
        let hacker_ip = utils::get_random_ip(&mut self.rng);
        let mut last_when = when.clone();
        for user in user_list {
            let new_ip = utils::get_random_ip(&mut self.rng);
            last_when = self.hacker_attempts_login(
                &mut last_when,
                if vary_ips { &new_ip } else { &hacker_ip },
                &user,
            );
        }
        return (hacker_ip, last_when);
    }
    fn hacker_attempts_login(
        &mut self,
        when: &mut NaiveDateTime,
        source_ip: &String,
        username: &String,
    ) -> NaiveDateTime {
        // mean stdev
        let normal = Normal::new(0.35, 0.5).unwrap();
        return self.attempt_login(
            when,
            &source_ip,
            &username,
            normal.sample(&mut rand::thread_rng()),
            self.attacker_success_probs.clone(), // TODO: need to remove clone
        );
    }
    fn attempt_login(
        &mut self,
        when: &mut NaiveDateTime,
        source_ip: &String,
        username: &String,
        username_accuracy: f64,
        success_likelihoods: Vec<f64>,
    ) -> NaiveDateTime {
        let user_list: Vec<String> = self.userbase.keys().cloned().collect();
        let mut login_user = username.clone();
        if self.rng.gen::<f64>() > username_accuracy {
            // Incorrect username is taken
            login_user = self.distort_username(login_user);
            info!("Distorted username - {}", login_user);
        }
        if !self.locked_accounts.contains(&login_user) {
            let tries = success_likelihoods.len();
            for index in 0..cmp::min(tries, ATTEMPTS_BEFORE_LOCKOUT) {
                *when = *when + Duration::seconds(1);
                if !user_list.contains(&login_user) {
                    self.logs.push(Log {
                        datetime: when.to_string(),
                        source_ip: source_ip.clone(),
                        username: login_user.clone(),
                        success: false,
                        failure_reason: Some(FailureReason::ErrorWrongUsername),
                    });
                    if self.rng.gen::<f64>() <= username_accuracy {
                        login_user = username.clone();
                    }
                    continue;
                }
                if self.rng.gen::<f64>() <= success_likelihoods[index] {
                    self.logs.push(Log {
                        datetime: when.to_string(),
                        source_ip: source_ip.clone(),
                        username: login_user.clone(),
                        success: true,
                        failure_reason: None,
                    });
                    break;
                } else {
                    self.logs.push(Log {
                        datetime: when.to_string(),
                        source_ip: source_ip.clone(),
                        username: login_user.clone(),
                        success: false,
                        failure_reason: Some(FailureReason::ErrorWrongPassword),
                    })
                }
            }
        } else {
            self.logs.push(Log {
                datetime: when.to_string(),
                source_ip: source_ip.clone(),
                username: login_user.clone(),
                success: false,
                failure_reason: Some(FailureReason::ErrorAccountLocked),
            })
        }
        if self.rng.gen::<f64>() >= 0.5 {
            self.locked_accounts.pop();
        }
        return when.clone();
    }
    fn distort_username(&mut self, username: String) -> String {
        let mut distorted_username = username.clone(); // avoid mutable borrows
        let index = self.rng.gen_range(0..username.len() - 1);
        if self.rng.gen::<f64>() < 0.5 {
            distorted_username.remove(index);
            distorted_username
        } else {
            let random_char = self.rng.gen_range('a'..'z');
            distorted_username.insert(index, random_char);
            distorted_username
        }
    }
}
