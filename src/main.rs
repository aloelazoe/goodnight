use tray_item::*;
use directories::{ProjectDirs};
use std::{error::Error, path::PathBuf, process::Command};
use serde::{Serialize, Deserialize};
use confy::{load_path, store_path};
use chrono::{Local, Timelike, Duration, NaiveTime};

#[link(name = "ApplicationServices", kind = "framework")]
extern {
    fn CGDisplayUsesForceToGray() -> bool;
    fn CGDisplayForceToGray(forceToGray: bool);
}

#[derive(Serialize, Deserialize, Debug)]
struct Time {
    hours: u32,
    minutes: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    start: Time,
    end: Time,
}

impl ::std::default::Default for Config {
    fn default() -> Self {
        Self { start: Time {hours: 1, minutes: 30}, end: Time {hours: 10, minutes: 0} }
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

    let time = Local::now().time();
    println!("time is: {}", &time);
    
    let nighttime = is_nighttime(time, config);
    dbg!(nighttime);
    set_grayscale(nighttime);

    set_up_tray(config_path);

    Ok(())
}

fn is_nighttime(time: NaiveTime, config: Config) -> bool {
    // convert everything to seconds from midnight
    let start = config.start.hours * 60 * 60 + config.start.minutes * 60;
    let end = config.end.hours * 60 * 60 + config.end.minutes * 60;
    let time = time.num_seconds_from_midnight();
    let same_day = start < end;

    if same_day {
        time > start && time < end
    } else {
        time > start || time < end
    }
}

fn seconds_until_next_switch(time: NaiveTime, config: Config) -> u32 {
    unimplemented!();
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

fn set_up_tray(config_path: PathBuf) {
    // 😴🌚☾☀︎
    let mut tray = TrayItem::new("🌚", "").unwrap();
    tray.add_label("gray screen for gay babes").unwrap();

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
    let config = Config { start: Time {hours: 1, minutes: 30}, end: Time {hours: 10, minutes: 0}};
    let time = NaiveTime::from_hms(0, 0, 0);
    assert_eq!(is_nighttime(time, config), false);
}

#[test]
fn same_day2() {
    let config = Config { start: Time {hours: 1, minutes: 30}, end: Time {hours: 10, minutes: 0}};
    let time = NaiveTime::from_hms(3, 0, 0);
    assert_eq!(is_nighttime(time, config), true);
}

#[test]
fn same_day3() {
    let config = Config { start: Time {hours: 1, minutes: 30}, end: Time {hours: 10, minutes: 0}};
    let time = NaiveTime::from_hms(12, 0, 0);
    assert_eq!(is_nighttime(time, config), false);
}

#[test]
fn same_day4() {
    let config = Config { start: Time {hours: 1, minutes: 30}, end: Time {hours: 10, minutes: 0}};
    let time = NaiveTime::from_hms(18, 0, 0);
    assert_eq!(is_nighttime(time, config), false);
}

#[test]
fn new_day5() {
    let config = Config { start: Time {hours: 22, minutes: 0}, end: Time {hours: 7, minutes: 0}};
    let time = NaiveTime::from_hms(18, 0, 0);
    assert_eq!(is_nighttime(time, config), false);
}

#[test]
fn new_day6() {
    let config = Config { start: Time {hours: 22, minutes: 0}, end: Time {hours: 7, minutes: 0}};
    let time = NaiveTime::from_hms(23, 0, 0);
    assert_eq!(is_nighttime(time, config), true);
}

#[test]
fn new_day7() {
    let config = Config { start: Time {hours: 22, minutes: 0}, end: Time {hours: 7, minutes: 0}};
    let time = NaiveTime::from_hms(0, 0, 0);
    assert_eq!(is_nighttime(time, config), true);
}


#[test]
fn new_day8() {
    let config = Config { start: Time {hours: 22, minutes: 0}, end: Time {hours: 7, minutes: 0}};
    let time = NaiveTime::from_hms(6, 0, 0);
    assert_eq!(is_nighttime(time, config), true);
}

#[test]
fn new_day9() {
    let config = Config { start: Time {hours: 22, minutes: 0}, end: Time {hours: 7, minutes: 0}};
    let time = NaiveTime::from_hms(8, 0, 0);
    assert_eq!(is_nighttime(time, config), false);
}

#[test]
fn new_day10() {
    let config = Config { start: Time {hours: 22, minutes: 0}, end: Time {hours: 7, minutes: 0}};
    let time = NaiveTime::from_hms(18, 0, 0);
    assert_eq!(is_nighttime(time, config), false);
}

#[test]
fn new_day11() {
    let config = Config { start: Time {hours: 22, minutes: 0}, end: Time {hours: 7, minutes: 0}};
    let time = NaiveTime::from_hms(18, 0, 0);
    assert_eq!(is_nighttime(time, config), false);
}
