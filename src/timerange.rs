use std::fmt;
use serde::{Serialize, Deserialize};
use chrono::{NaiveTime, offset::TimeZone, DateTime, Duration};

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

    #[allow(unused)]
    /// return time between given `time` and range boundary that would come sooner
    pub fn time_until_boundary_from(self, time: NaiveTime) -> NaiveTime {
        NaiveTime::from_hms(0, 0, 0) + (self.next_boundary_from(time) - time)
    }

    /// return duration between given `time` and range boundary that would come sooner
    pub fn duration_until_boundary_from(self, time: NaiveTime) -> Duration {
        let mut dur = self.next_boundary_from(time) - time;
        // account for negative duration when boundary is on the next day
        if dur < Duration::zero() { dur = dur + Duration::days(1) };
        dur
    }

    pub fn did_cross_boundary<Tz: TimeZone>(self, since: DateTime<Tz>, until: DateTime<Tz>) -> bool
        where <Tz as TimeZone>::Offset: Copy,
    {
        (until - since) > self.duration_until_boundary_from(since.time())
    }
}

impl fmt::Display for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.start.format("%H:%M"), self.end.format("%H:%M"))
    }
}

#[cfg(test)]
mod test {
    use chrono::Timelike;

    use super::*;

    #[test]
    fn same_day_1() {
        let nighttime = TimeRange::from_hmhm(1, 30, 10, 0);
        let time = NaiveTime::from_hms(0, 0, 0);
        assert_eq!(nighttime.includes(time), false);
        assert_eq!(nighttime.time_until_boundary_from(time), NaiveTime::from_hms(1, 30, 0));
        assert_eq!(nighttime.duration_until_boundary_from(time), Duration::minutes(90));
    }

    #[test]
    fn same_day_2() {
        let nighttime = TimeRange::from_hmhm(1, 30, 10, 0);
        let time = NaiveTime::from_hms(3, 0, 0);
        assert_eq!(nighttime.includes(time), true);
        assert_eq!(nighttime.time_until_boundary_from(time), NaiveTime::from_hms(7, 0, 0));
        assert_eq!(nighttime.duration_until_boundary_from(time), Duration::hours(7));
    }

    #[test]
    fn same_day_3() {
        let nighttime = TimeRange::from_hmhm(1, 30, 10, 0);
        let time = NaiveTime::from_hms(12, 0, 0);
        assert_eq!(nighttime.includes(time), false);
        assert_eq!(nighttime.time_until_boundary_from(time), NaiveTime::from_hms(13, 30, 0));
        assert_eq!(
            (NaiveTime::from_hms(0, 0, 0) + nighttime.duration_until_boundary_from(time)).num_seconds_from_midnight() / 60,
            30 + (13 * 60)
        );
    }

    #[test]
    fn same_day_4() {
        let nighttime = TimeRange::from_hmhm(1, 30, 10, 0);
        let time = NaiveTime::from_hms(18, 0, 0);
        assert_eq!(nighttime.includes(time), false);
        assert_eq!(nighttime.time_until_boundary_from(time), NaiveTime::from_hms(7, 30, 0));
        assert_eq!(nighttime.duration_until_boundary_from(time), Duration::minutes(30 + (7 * 60)));
    }

    #[test]
    fn new_day_5() {
        let nighttime = TimeRange::from_hmhm(22, 0, 7, 0);
        let time = NaiveTime::from_hms(18, 0, 0);
        assert_eq!(nighttime.includes(time), false);
        assert_eq!(nighttime.time_until_boundary_from(time), NaiveTime::from_hms(4, 0, 0));
    }

    #[test]
    fn new_day_6() {
        let nighttime = TimeRange::from_hmhm(22, 0, 7, 0);
        let time = NaiveTime::from_hms(23, 0, 0);
        assert_eq!(nighttime.includes(time), true);
        assert_eq!(nighttime.time_until_boundary_from(time), NaiveTime::from_hms(8, 0, 0));
    }

    #[test]
    fn new_day_7() {
        let nighttime = TimeRange::from_hmhm(22, 0, 7, 0);
        let time = NaiveTime::from_hms(0, 0, 0);
        assert_eq!(nighttime.includes(time), true);
        assert_eq!(nighttime.time_until_boundary_from(time), NaiveTime::from_hms(7, 0, 0));
    }


    #[test]
    fn new_day_8() {
        let nighttime = TimeRange::from_hmhm(22, 0, 7, 0);
        let time = NaiveTime::from_hms(6, 0, 0);
        assert_eq!(nighttime.includes(time), true);
        assert_eq!(nighttime.time_until_boundary_from(time), NaiveTime::from_hms(1, 0, 0));
    }

    #[test]
    fn new_day_9() {
        let nighttime = TimeRange::from_hmhm(22, 0, 7, 0);
        let time = NaiveTime::from_hms(8, 0, 0);
        assert_eq!(nighttime.includes(time), false);
        assert_eq!(nighttime.time_until_boundary_from(time), NaiveTime::from_hms(14, 0, 0));
    }

    #[test]
    fn boundary_1() {
        let nighttime = TimeRange::from_hmhm(22, 0, 7, 0);
        let since = DateTime::parse_from_rfc3339("2021-01-01T12:30:00-00:00").unwrap();
        let until = DateTime::parse_from_rfc3339("2021-01-01T12:31:00-00:00").unwrap();
        assert_eq!(nighttime.did_cross_boundary(since, until), false);
    }

    #[test]
    fn boundary_2() {
        let nighttime = TimeRange::from_hmhm(22, 0, 7, 0);
        let since = DateTime::parse_from_rfc3339("2021-01-01T12:30:00-00:00").unwrap();
        let until = DateTime::parse_from_rfc3339("2021-01-01T23:30:00-00:00").unwrap();
        assert_eq!(nighttime.did_cross_boundary(since, until), true);
    }

    #[test]
    fn boundary_3() {
        let nighttime = TimeRange::from_hmhm(22, 0, 7, 0);
        let since = DateTime::parse_from_rfc3339("2021-01-01T12:30:00-00:00").unwrap();
        let until = DateTime::parse_from_rfc3339("2021-03-01T12:31:00-00:00").unwrap();
        assert_eq!(nighttime.did_cross_boundary(since, until), true);
    }

    #[test]
    fn boundary_3_reverse() {
        let nighttime = TimeRange::from_hmhm(22, 0, 7, 0);
        let since = DateTime::parse_from_rfc3339("2021-01-01T12:30:00-00:00").unwrap();
        let until = DateTime::parse_from_rfc3339("2021-03-01T12:31:00-00:00").unwrap();
        assert_eq!(nighttime.did_cross_boundary(until, since), false);
    }

    #[test]
    fn boundary_same_minute_1() {
        let nighttime = TimeRange::from_hmhm(0, 25, 0, 25);
        let since = DateTime::parse_from_rfc3339("2021-01-01T00:26:00-00:00").unwrap();
        let until = DateTime::parse_from_rfc3339("2021-01-01T00:26:05-00:00").unwrap();
        assert_eq!(nighttime.did_cross_boundary(since, until), false);
    }

    #[test]
    fn boundary_same_minute_2() {
        let nighttime = TimeRange {
            start: NaiveTime::from_hms(0, 25, 0),
            end: NaiveTime::from_hms(0, 25, 10),
        };
        let since = DateTime::parse_from_rfc3339("2021-01-01T00:26:00-00:00").unwrap();
        let until = DateTime::parse_from_rfc3339("2021-01-01T00:26:05-00:00").unwrap();
        assert_eq!(nighttime.did_cross_boundary(since, until), false);
    }

    #[test]
    fn boundary_almost_same_minute_1() {
        let nighttime = TimeRange {
            start: NaiveTime::from_hms(0, 25, 0),
            end: NaiveTime::from_hms(0, 26, 0),
        };
        let since = DateTime::parse_from_rfc3339("2021-01-01T00:25:59-00:00").unwrap();
        let until = DateTime::parse_from_rfc3339("2021-01-01T00:26:05-00:00").unwrap();
        assert_eq!(nighttime.did_cross_boundary(since, until), true);
    }

    #[test]
    fn boundary_almost_same_minute_2() {
        let nighttime = TimeRange {
            start: NaiveTime::from_hms(0, 25, 0),
            end: NaiveTime::from_hms(0, 26, 0),
        };
        let since = DateTime::parse_from_rfc3339("2021-01-01T00:26:01-00:00").unwrap();
        let until = DateTime::parse_from_rfc3339("2021-01-01T00:26:05-00:00").unwrap();
        assert_eq!(nighttime.did_cross_boundary(since, until), false);
    }

    #[test]
    fn boundary_almost_same_minute_3() {
        let nighttime = TimeRange {
            start: NaiveTime::from_hms(0, 25, 0),
            end: NaiveTime::from_hms(0, 26, 0),
        };
        let since = DateTime::parse_from_rfc3339("2021-01-01T00:30:00-00:00").unwrap();
        let until = DateTime::parse_from_rfc3339("2021-01-01T00:30:05-00:00").unwrap();
        assert_eq!(nighttime.did_cross_boundary(since, until), false);
    }

    #[test]
    fn boundary_almost_same_minute_4() {
        let nighttime = TimeRange {
            start: NaiveTime::from_hms(0, 0, 0),
            end: NaiveTime::from_hms(0, 26, 0),
        };
        let since = DateTime::parse_from_rfc3339("2021-01-01T00:30:00-00:00").unwrap();
        let until = DateTime::parse_from_rfc3339("2021-01-01T00:30:05-00:00").unwrap();
        assert_eq!(nighttime.did_cross_boundary(since, until), false);
    }

    #[test]
    fn boundary_almost_same_minute_5() {
        let nighttime = TimeRange {
            start: NaiveTime::from_hms(0, 0, 0),
            end: NaiveTime::from_hms(0, 26, 0),
        };
        let since = DateTime::parse_from_rfc3339("2021-01-01T00:30:00-00:00").unwrap();
        let until = DateTime::parse_from_rfc3339("2021-01-01T04:30:05-00:00").unwrap();
        assert_eq!(nighttime.did_cross_boundary(since, until), false);
    }

    #[test]
    fn boundary_almost_same_minute_6() {
        let nighttime = TimeRange {
            start: NaiveTime::from_hms(0, 22, 0),
            end: NaiveTime::from_hms(0, 26, 0),
        };
        let since = DateTime::parse_from_rfc3339("2021-01-01T00:30:00-00:00").unwrap();
        let until = DateTime::parse_from_rfc3339("2021-01-01T08:30:05-00:00").unwrap();
        assert_eq!(nighttime.did_cross_boundary(since, until), false);
    }

    #[test]
    fn boundary_quite_a_different_minute_1() {
        let nighttime = TimeRange {
            start: NaiveTime::from_hms(22, 0, 0),
            end: NaiveTime::from_hms(0, 26, 0),
        };
        let since = DateTime::parse_from_rfc3339("2021-01-01T00:30:00-00:00").unwrap();
        let until = DateTime::parse_from_rfc3339("2021-01-01T08:30:05-00:00").unwrap();
        assert_eq!(nighttime.did_cross_boundary(since, until), false);
    }

    #[test]
    fn boundary_within_hour_different_days() {
        let nighttime = TimeRange {
            start: NaiveTime::from_hms(23, 26, 0),
            end: NaiveTime::from_hms(0, 26, 0),
        };
        let since = DateTime::parse_from_rfc3339("2021-01-01T00:30:00-00:00").unwrap();
        let until = DateTime::parse_from_rfc3339("2021-01-01T08:30:05-00:00").unwrap();
        assert_eq!(nighttime.did_cross_boundary(since, until), false);
    }

    #[test]
    fn boundary_within_halfhour_different_days() {
        let nighttime = TimeRange {
            start: NaiveTime::from_hms(23, 56, 0),
            end: NaiveTime::from_hms(0, 26, 0),
        };
        let since = DateTime::parse_from_rfc3339("2021-01-01T00:30:00-00:00").unwrap();
        let until = DateTime::parse_from_rfc3339("2021-01-01T08:30:05-00:00").unwrap();
        assert_eq!(nighttime.did_cross_boundary(since, until), false);
    }

    #[test]
    fn boundary_within_20_minutes_same_day() {
        let nighttime = TimeRange {
            start: NaiveTime::from_hms(0, 6, 0),
            end: NaiveTime::from_hms(0, 26, 0),
        };
        let since = DateTime::parse_from_rfc3339("2021-01-01T00:30:00-00:00").unwrap();
        let until = DateTime::parse_from_rfc3339("2021-01-01T08:30:05-00:00").unwrap();
        assert_eq!(nighttime.did_cross_boundary(since, until), false);
    }

    #[test]
    fn boundary_within_25_minutes_same_day() {
        let nighttime = TimeRange {
            start: NaiveTime::from_hms(0, 1, 0),
            end: NaiveTime::from_hms(0, 26, 0),
        };
        let since = DateTime::parse_from_rfc3339("2021-01-01T00:30:00-00:00").unwrap();
        let until = DateTime::parse_from_rfc3339("2021-01-01T08:30:05-00:00").unwrap();
        assert_eq!(nighttime.did_cross_boundary(since, until), false);
    }

    #[test]
    fn boundary_within_27_minutes_different_days() {
        let nighttime = TimeRange {
            start: NaiveTime::from_hms(23, 59, 0),
            end: NaiveTime::from_hms(0, 26, 0),
        };
        let since = DateTime::parse_from_rfc3339("2021-01-01T00:30:00-00:00").unwrap();
        let until = DateTime::parse_from_rfc3339("2021-01-01T08:30:05-00:00").unwrap();
        assert_eq!(nighttime.did_cross_boundary(since, until), false);
    }

    #[test]
    fn boundary_within_27_minutes_same_day() {
        let nighttime = TimeRange {
            start: NaiveTime::from_hms(0, 0, 0),
            end: NaiveTime::from_hms(0, 27, 0),
        };
        let since = DateTime::parse_from_rfc3339("2021-01-01T00:30:00-00:00").unwrap();
        let until = DateTime::parse_from_rfc3339("2021-01-01T08:30:05-00:00").unwrap();
        assert_eq!(nighttime.did_cross_boundary(since, until), false);
    }

    #[test]
    fn boundary_within_27_minutes_same_day_alt() {
        let nighttime = TimeRange {
            start: NaiveTime::from_hms(0, 0, 0),
            end: NaiveTime::from_hms(0, 27, 0),
        };
        let since = DateTime::parse_from_rfc3339("2021-01-01T04:30:00-00:00").unwrap();
        let until = DateTime::parse_from_rfc3339("2021-01-01T10:30:05-00:00").unwrap();
        assert_eq!(nighttime.did_cross_boundary(since, until), false);
    }

    #[test]
    fn boundary_within_4_hours_same_day() {
        let nighttime = TimeRange {
            start: NaiveTime::from_hms(0, 0, 0),
            end: NaiveTime::from_hms(4, 0, 0),
        };
        let since = DateTime::parse_from_rfc3339("2021-01-01T04:30:00-00:00").unwrap();
        let until = DateTime::parse_from_rfc3339("2021-01-01T10:30:05-00:00").unwrap();
        assert_eq!(nighttime.did_cross_boundary(since, until), false);
    }

    #[test]
    fn boundary_new_day_1() {
        let nighttime = TimeRange::from_hmhm(22, 0, 7, 0);
        let time = NaiveTime::from_hms(18, 0, 0);
        assert_eq!(nighttime.includes(time), false);
        assert_eq!(nighttime.time_until_boundary_from(time), NaiveTime::from_hms(4, 0, 0));
    }
}
