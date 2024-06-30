use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Time(u32, u32);

impl Time {
    pub fn new(hours: u32, minutes: u32) -> Self {
        Self(hours, minutes)
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
