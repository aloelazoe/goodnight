use tray_item::*;
use directories::{ProjectDirs};
use std::{error::Error, path::PathBuf, process::Command};
use serde::{Serialize, Deserialize};
use confy::{load_path, store_path};
use chrono::{Local};

mod timerange;
use timerange::TimeRange;

#[link(name = "ApplicationServices", kind = "framework")]
extern {
    fn CGDisplayUsesForceToGray() -> bool;
    fn CGDisplayForceToGray(forceToGray: bool);
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
