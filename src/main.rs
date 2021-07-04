use tray_item::*;
use directories::{ProjectDirs};
use std::{error::Error, path::PathBuf, process::Command, fmt};
use serde::{Serialize, Deserialize};
use confy::{load_path, store_path};
use chrono::{Local, NaiveTime};

#[link(name = "ApplicationServices", kind = "framework")]
extern {
    fn CGDisplayUsesForceToGray() -> bool;
    fn CGDisplayForceToGray(forceToGray: bool);
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct TimeRange {
    start: NaiveTime,
    end: NaiveTime,
}

impl TimeRange {
    /// create time range defined as start and end boundary in hours and minutes
    fn from_hmhm(start_h: u32, start_m: u32, end_h: u32, end_m: u32) -> Self {
        Self {
            start: NaiveTime::from_hms(start_h, start_m, 0),
            end: NaiveTime::from_hms(end_h, end_m, 0),
        }
    }

    /// check if given `time` is within this time range
    fn includes(self, time: NaiveTime) -> bool {
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
    fn next_boundary_from(self, time: NaiveTime) -> NaiveTime {
        if self.includes(time) {self.end} else {self.start}
    }

    /// return time between given `time` and range boundary that would come sooner
    fn time_until_boundary_from(self, time: NaiveTime) -> NaiveTime {
        NaiveTime::from_hms(0, 0, 0) + (self.next_boundary_from(time) - time)
    }
}

impl fmt::Display for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.start.format("%H:%M"), self.end.format("%H:%M"))
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    nighttime: TimeRange,
}

impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            nighttime: TimeRange::from_hmhm(1, 30, 11, 00),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let project_dirs = ProjectDirs::from("", "",  "nighttime").unwrap();
    let mut config_path = project_dirs.config_dir().to_owned();
    config_path.push("config.yaml");
    dbg!(&config_path);

    let config: Config = load_path(&config_path).unwrap_or_else(|err| {
        println!("error when parsing config file: {}", err);
        println!("overwriting with default config");
        let config = Config::default();
        store_path(&config_path, &config).expect("can't write default configuration file");
        config
    });
    dbg!(&config);
    let nighttime = config.nighttime;
    println!("night time is: {}", nighttime);

    let now = Local::now().time();
    println!("current time is: {}", &now.format("%H:%M"));
    
    let is_nighttime = nighttime.includes(now);
    dbg!(nighttime);
    let next_switch_time = nighttime.next_boundary_from(now);
    dbg!(next_switch_time);
    let time_until_switch = nighttime.time_until_boundary_from(now);
    dbg!(time_until_switch);

    // todo: sleep for the amount of time until next switch

    set_grayscale(is_nighttime);

    start_tray(config_path);

    Ok(())
}

fn set_grayscale(on: bool) {
    unsafe {
        CGDisplayForceToGray(on);
    }
}

fn toggle_grayscale() {
    let gray = unsafe {
        CGDisplayUsesForceToGray()
    };
    println!("display is in grayscale mode: {}. now switching...", gray);
    unsafe {
        CGDisplayForceToGray(!gray);
    }
}

fn start_tray(config_path: PathBuf) {
    // 😴🌚☾☀︎
    let mut tray = TrayItem::new("🌚", "").unwrap();
    tray.add_label("✨GRAY SCREEN FOR GAY BABES✨").unwrap();

    // todo: display current settings in the menu item
    // but tray_item library is not enough for that, would have to
    // use macos api directly

    tray.add_menu_item("edit settings", move || {
        Command::new("open")
            .arg(&config_path)
            .output()
            .expect("failed to open config file in system default application");
    }).unwrap();

    tray.add_menu_item("restart with new settings", || {
        unimplemented!();
    }).unwrap();

    tray.add_menu_item("toggle grayscale", || {
        toggle_grayscale();
    }).unwrap();

    #[allow(unused_mut)]
    let mut inner = tray.inner_mut();
    inner.add_quit_item("quit");
    inner.display();
}

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
