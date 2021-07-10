use std::{
    path::PathBuf,
    process::Command,
};
// use cocoa::base::id;
// use objc::runtime::{Object, Sel};
use tray_item::TrayItem;
use winit::{
    event_loop::{EventLoopProxy},
};

use crate::config::Config;
use crate::grayscale::{is_grayscale, set_grayscale};
use crate::CustomEvent;

#[allow(unused_variables)]
pub fn create_tray(config_path: PathBuf, config: &Config, was_grayscale: bool, loop_proxy: EventLoopProxy<CustomEvent>) {
    // 😴🌚☾☀︎
    let mut tray = TrayItem::new(&config.title, "").unwrap();
    #[allow(unused_mut)]
    let mut tray = tray.inner_mut();
    tray.add_label(&format!("✨GRAY SCREEN FOR GAY BABES {}✨", &config.nighttime)).unwrap();
    #[cfg(debug_assertions)]
    tray.add_label(&format!("debug mode")).unwrap();

    let sender = loop_proxy.clone();
    tray.add_menu_item("toggle grayscale", move || {
        let new_grayscale = !is_grayscale();
        set_grayscale(new_grayscale);
        &sender.send_event(CustomEvent::GrayscaleToggle(new_grayscale));
    }).unwrap();

    tray.add_menu_item("open config", move || {
        Command::new("open")
            .arg(&config_path)
            .output()
            .expect("failed to open config file in system default application");
    }).unwrap();

    let sender = loop_proxy.clone();
    tray.add_menu_item("create window", move || {
        &sender.send_event(CustomEvent::CreateWindow);
    }).unwrap();

    let sender = loop_proxy.clone();
    tray.add_menu_item("destroy window", move || {
        &sender.send_event(CustomEvent::DestroyWindow);
    }).unwrap();

    let sender = loop_proxy.clone();
    tray.add_menu_item("quit", move || {
        &sender.send_event(CustomEvent::Exit);
    }).unwrap();

    tray.add_quit_item("terminate");

    tray.display();
}
