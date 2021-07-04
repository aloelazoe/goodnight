use tray_item::TrayItem;
use directories::{ProjectDirs};
use std::{
    error::Error,
    path::PathBuf,
    process::Command,
    thread,
    time::Duration
};
use serde::{Serialize, Deserialize};
use confy::{load_path, store_path};
use chrono::{Local, Timelike};

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
        store_path(&config_path, &config)
            .expect("can't write default configuration file");
        config
    });
    dbg!(&config);
    let nighttime = config.nighttime;
    println!("\nnight time is: {}", nighttime);
    
    thread::spawn(move || {
        loop {
            let now = Local::now().time();
            // println!("current time is: {}", &now.format("%H:%M"));
            println!("\ncurrent time is: {}", &now);
            let is_nighttime = nighttime.includes(now);
            dbg!(is_nighttime);

            set_grayscale(is_nighttime);

            let next_switch_time = nighttime.next_boundary_from(now);
            println!(
                "next time toggle grayscale at: {}",
                // &next_switch_time.format("%H:%M")
                &next_switch_time
            );

            let time_until_switch = nighttime.time_until_boundary_from(now);
            println!(
                "time until next toggle: {}",
                // &time_until_switch.format("%H:%M")
                &time_until_switch
            );
            let sleep_duration = 
                Duration::from_secs(time_until_switch.num_seconds_from_midnight().into()) +
                Duration::from_nanos(time_until_switch.nanosecond().into());
            dbg!(sleep_duration);

            thread::sleep(sleep_duration);
        }
    });

    start_tray(config_path, nighttime);

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

fn start_tray(config_path: PathBuf, nighttime: TimeRange) {
    // 😴🌚☾☀︎
    let mut tray = TrayItem::new("🌚", "").unwrap();
    tray.add_label(&format!("✨GRAY SCREEN FOR GAY BABES {}✨", nighttime)).unwrap();

    // todo: display current settings in the menu item
    // but tray_item library is not enough for that, would have to
    // use macos api directly

    tray.add_menu_item("edit settings - needs restart", move || {
        Command::new("open")
            .arg(&config_path)
            .output()
            .expect("failed to open config file in system default application");
    }).unwrap();

    tray.add_menu_item("toggle grayscale", || {
        toggle_grayscale();
    }).unwrap();

    #[allow(unused_mut)]
    let mut inner = tray.inner_mut();
    inner.add_quit_item("quit");
    inner.display();
}
