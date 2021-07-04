use std::fmt;
use serde::{Serialize, Deserialize};
use chrono::{NaiveTime};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct TimeRange {
    start: NaiveTime,
    end: NaiveTime,
}

impl TimeRange {
    /// create time range defined as start and end boundary in hours and minutes
    pub fn from_hmhm(start_h: u32, start_m: u32, end_h: u32, end_m: u32) -> Self {
        Self {
            start: NaiveTime::from_hms(start_h, start_m, 0),
            end: NaiveTime::from_hms(end_h, end_m, 0),
        }
    }

    /// check if given `time` is within this time range
    pub fn includes(self, time: NaiveTime) -> bool {
        let Self {start, end} = self;
        let same_day = start < end;
    
        if same_day {
            time > start && time < end
        } else {
            time > start || time < end
        }
    }

    /// return either the start or the end of this time range,
    /// depending on which would come sooner relative to the given `time`
    pub fn next_boundary_from(self, time: NaiveTime) -> NaiveTime {
        if self.includes(time) {self.end} else {self.start}
    }

    /// return time between given `time` and range boundary that would come sooner
    pub fn time_until_boundary_from(self, time: NaiveTime) -> NaiveTime {
        NaiveTime::from_hms(0, 0, 0) + (self.next_boundary_from(time) - time)
    }
}

impl fmt::Display for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.start.format("%H:%M"), self.end.format("%H:%M"))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn same_day1() {
        let nighttime = TimeRange::from_hmhm(1, 30, 10, 0);
        let time = NaiveTime::from_hms(0, 0, 0);
        assert_eq!(nighttime.includes(time), false);
        assert_eq!(nighttime.time_until_boundary_from(time), NaiveTime::from_hms(1, 30, 0));
    }

    #[test]
    fn same_day2() {
        let nighttime = TimeRange::from_hmhm(1, 30, 10, 0);
        let time = NaiveTime::from_hms(3, 0, 0);
        assert_eq!(nighttime.includes(time), true);
        assert_eq!(nighttime.time_until_boundary_from(time), NaiveTime::from_hms(7, 0, 0));
    }

    #[test]
    fn same_day3() {
        let nighttime = TimeRange::from_hmhm(1, 30, 10, 0);
        let time = NaiveTime::from_hms(12, 0, 0);
        assert_eq!(nighttime.includes(time), false);
        assert_eq!(nighttime.time_until_boundary_from(time), NaiveTime::from_hms(13, 30, 0));
    }

    #[test]
    fn same_day4() {
        let nighttime = TimeRange::from_hmhm(1, 30, 10, 0);
        let time = NaiveTime::from_hms(18, 0, 0);
        assert_eq!(nighttime.includes(time), false);
        assert_eq!(nighttime.time_until_boundary_from(time), NaiveTime::from_hms(7, 30, 0));
    }

    #[test]
    fn new_day5() {
        let nighttime = TimeRange::from_hmhm(22, 0, 7, 0);
        let time = NaiveTime::from_hms(18, 0, 0);
        assert_eq!(nighttime.includes(time), false);
        assert_eq!(nighttime.time_until_boundary_from(time), NaiveTime::from_hms(4, 0, 0));
    }

    #[test]
    fn new_day6() {
        let nighttime = TimeRange::from_hmhm(22, 0, 7, 0);
        let time = NaiveTime::from_hms(23, 0, 0);
        assert_eq!(nighttime.includes(time), true);
        assert_eq!(nighttime.time_until_boundary_from(time), NaiveTime::from_hms(8, 0, 0));
    }

    #[test]
    fn new_day7() {
        let nighttime = TimeRange::from_hmhm(22, 0, 7, 0);
        let time = NaiveTime::from_hms(0, 0, 0);
        assert_eq!(nighttime.includes(time), true);
        assert_eq!(nighttime.time_until_boundary_from(time), NaiveTime::from_hms(7, 0, 0));
    }


    #[test]
    fn new_day8() {
        let nighttime = TimeRange::from_hmhm(22, 0, 7, 0);
        let time = NaiveTime::from_hms(6, 0, 0);
        assert_eq!(nighttime.includes(time), true);
        assert_eq!(nighttime.time_until_boundary_from(time), NaiveTime::from_hms(1, 0, 0));
    }

    #[test]
    fn new_day9() {
        let nighttime = TimeRange::from_hmhm(22, 0, 7, 0);
        let time = NaiveTime::from_hms(8, 0, 0);
        assert_eq!(nighttime.includes(time), false);
        assert_eq!(nighttime.time_until_boundary_from(time), NaiveTime::from_hms(14, 0, 0));
    }
}