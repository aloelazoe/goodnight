use serde::{Serialize, Deserialize};
use crate::timerange::TimeRange;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub nighttime: TimeRange,
    pub loop_seconds: u64,
    pub title: String,
}

impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            nighttime: TimeRange::from_hmhm(0, 30, 10, 00),
            loop_seconds: 60,
            title: "🌚".to_owned(),
        }
    }
}
