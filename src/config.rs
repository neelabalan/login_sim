use chrono::{NaiveDateTime, Timelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::constants;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Config {
    pub start_date: String,
    pub end_date: String,
    pub seed: u64,
    pub first_names: Vec<String>,
    pub last_names: Vec<String>,
    pub output: HashMap<String, String>,
    pub attacker_success_probabilities: Vec<f32>,
    pub valid_user_success_probabilities: Vec<f32>,
    pub time_periods: Vec<TimePeriod>,
    pub attack_probability: f32,
    pub vary_ips: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct TimePeriod {
    pub name: String,
    pub days: Vec<String>,
    pub hour_range: String,
    pub distribution: Distribution,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Distribution {
    #[serde(rename = "type")]
    pub dist_type: String,
    pub params: HashMap<String, f32>,
}

impl TimePeriod {
    pub fn parse_hour_range(&self) -> Result<(u8, u8), String> {
        let parts: Vec<&str> = self.hour_range.split("..").collect();
        if parts.len() != 2 {
            return Err("Invalid hour_range format, expected 'start..end'".to_string());
        }
        let start_hour: u8 = parts[0].parse().map_err(|_| "Invalid start hour")?;
        let end_hour: u8 = parts[1].parse().map_err(|_| "Invalid end hour")?;
        Ok((start_hour, end_hour))
    }

    pub fn matches(&self, when: NaiveDateTime) -> Result<bool, String> {
        use chrono::Datelike;

        let weekday_name = format!("{:?}", when.weekday());
        if !self.days.contains(&weekday_name) {
            return Ok(false);
        }

        let (start_hour, end_hour) = self.parse_hour_range()?;
        let hour = when.hour() as u8;

        let matches = if start_hour < end_hour {
            hour >= start_hour && hour < end_hour
        } else {
            hour >= start_hour || hour < end_hour
        };

        Ok(matches)
    }
}

impl Config {
    pub fn get_time_period(&self, when: NaiveDateTime) -> Result<Option<&TimePeriod>, String> {
        for period in self.time_periods.iter() {
            if period.matches(when)? {
                return Ok(Some(period));
            }
        }
        Ok(None)
    }
    pub fn get_hour_range(&self) -> Result<i64, chrono::format::ParseError> {
        Ok(
            (NaiveDateTime::parse_from_str(self.end_date.as_str(), constants::DATE_FORMAT)?
                - NaiveDateTime::parse_from_str(self.start_date.as_str(), constants::DATE_FORMAT)?)
            .num_hours(),
        )
    }
}
