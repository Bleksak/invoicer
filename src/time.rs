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
}
