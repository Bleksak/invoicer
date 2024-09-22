use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Time(
    u32,
    u32,
);

impl Time {
    pub fn new(hours: u32, minutes: u32) -> Self {
        Self(
            hours, minutes,
        )
    }

    pub fn hours(&self) -> u32 {
        self.0
    }

    pub fn minutes(&self) -> u32 {
        self.1
    }

    pub fn hour_multiplicator(&self) -> f64 {
        self.hours() as f64 + self.minutes() as f64 / 60.0
    }
}

impl FromStr for Time {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut parts = value.split(':');

        let hours = parts
            .next()
            .ok_or("No hours")?
            .parse()
            .or(Err("No hours"))?;
        let minutes = parts
            .next()
            .ok_or("No minutes")?
            .parse()
            .or(Err("No minutes"))?;

        Ok(
            Self::new(
                hours, minutes,
            ),
        )
    }
}
