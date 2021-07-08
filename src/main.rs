#[macro_use] extern crate cocoa;
#[macro_use] extern crate objc;

use cocoa::base::id;
use objc::runtime::{Object, Sel};

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
use chrono::{Local, TimeZone};

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
    loop_seconds: u64,
}

impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            nighttime: TimeRange::from_hmhm(1, 30, 11, 00),
            loop_seconds: 60,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let project_dirs = ProjectDirs::from("", "",  "nighttime").unwrap();
    let mut config_path = project_dirs.config_dir().to_owned();
    if cfg!(debug_assertions) {
        config_path.push("config.debug.yaml");
    } else {
        config_path.push("config.yaml");
    }
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
    let loop_frequency = Duration::from_secs(config.loop_seconds);
    // check if the screen is already in grayscale or not to revert to the
    // original setting when quitting the app
    let was_grayscale = is_grayscale();

    thread::spawn(move || {
        // don't reset manually set grayscale but only until next night time boundary
        // e.g. if you turn on grayscale earlier than nighttime starts we still turn it off in the morning
        // and if you turn off grayscale manually early in the morning we still turn it on at night
        // this should also account for cases when the previous loop iteration was the same time period as the current one
        // but we did cross the night time boundary in the real time, e.g. when laptop was asleep the whole day
        let mut previous = Local.timestamp(0, 0);

        loop {
            let now = Local::now();
            if nighttime.did_cross_boundary(previous, now) {
                let is_nighttime = nighttime.includes(Local::now().time());
                if is_nighttime != is_grayscale() {
                    set_grayscale(is_nighttime);
                }
            }
            previous = now;
            thread::sleep(loop_frequency);
        }
    });

    start_tray(config_path, nighttime, was_grayscale);

    Ok(())
}

fn set_grayscale(on: bool) {
    unsafe {
        CGDisplayForceToGray(on);
    }
}

fn is_grayscale() -> bool {
    unsafe {
        CGDisplayUsesForceToGray()
    }
}

fn toggle_grayscale() {
    set_grayscale(!is_grayscale());
}

fn start_tray(config_path: PathBuf, nighttime: TimeRange, was_grayscale: bool) {
    // 😴🌚☾☀︎
    let mut tray = TrayItem::new("🌚", "").unwrap();
    tray.add_label(&format!("✨GRAY SCREEN FOR GAY BABES {}✨", nighttime)).unwrap();
    #[cfg(debug_assertions)]
    tray.add_label(&format!("debug mode")).unwrap();
    // todo: display current settings in the menu item
    // but tray_item library is not enough for that, would have to
    // use macos api directly or another library

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
    
    // revert to the original grayscale setting when quitting the app
    extern fn on_app_should_terminate(this: &Object, _cmd: Sel, _notification: id) {
        let was_grayscale: bool = unsafe { *this.get_ivar("was_grayscale") };
        set_grayscale(was_grayscale);
    }
    unsafe {
        inner.set_app_delegate(delegate!("AppDelegate", {
            was_grayscale: bool = was_grayscale,
            (applicationWillTerminate:) => on_app_should_terminate as extern fn(&Object, Sel, id)
        }));
    }

    inner.display();
}
