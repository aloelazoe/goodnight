// this needs to be at the crate root
// #[macro_use] extern crate cocoa;
// #[macro_use] extern crate objc;

mod config;
mod timerange;
mod grayscale;
mod tray;

use directories::{ProjectDirs};
use std::{error::Error, thread, time::Duration, rc::Rc};
use confy::{load_path, store_path};
use chrono::{Local, TimeZone};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::macos::{
        EventLoopExtMacOS,
    },
    window::{Window, WindowBuilder},
    dpi::{LogicalSize, LogicalPosition},
};
use crate::config::Config;
use crate::grayscale::{is_grayscale, set_grayscale};
use crate::tray::add_status_bar_button;

#[derive(Debug)]
pub enum CustomEvent {
    GrayscaleToggle(bool),
    // maybe going to make it crossplatform in the future
    // positioning the window relative to the status bar/tray item is platform
    // dependent, so the suggested position is optional
    ToggleWindow (Option<LogicalPosition<f64>>),
    Exit,
}

fn main() -> Result<(), Box<dyn Error>> {
    let project_dirs = ProjectDirs::from("", "",  "goodnight").unwrap();
    let mut config_path = project_dirs.config_dir().to_owned();
    if cfg!(debug_assertions) {
        config_path.push("config.debug.yaml");
    } else {
        config_path.push("config.yaml");
    }
    dbg!(&config_path);

    // wrap config in Rc to ensure that raw pointer to it that we need to use
    // from objective-c stays valid after config gets moved into loop closure
    let config: Rc<Config> = Rc::new(load_path(&config_path).unwrap_or_else(|err| {
        println!("error when parsing config file: {}", err);
        println!("overwriting with default config");
        let config = Config::default();
        store_path(&config_path, &config)
            .expect("can't write default configuration file");
        config
    }));
    dbg!(&config);
    let nighttime = config.nighttime;
    let loop_frequency = Duration::from_secs(config.loop_seconds);
    // check if the screen is already in grayscale or not to revert to the
    // original setting when quitting the app if it wasn't toggled manually
    let mut was_grayscale = is_grayscale();

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

    let mut event_loop: EventLoop<CustomEvent> = EventLoop::with_user_event();
    event_loop.set_activation_policy(winit::platform::macos::ActivationPolicy::Accessory);
    event_loop.enable_default_menu_creation(false);

    // todo: how do i make sure proxy has a long enough lifetime so that the
    // raw pointer to it inside the objective-c object stays valid
    // for as long as program runs? i kinda can know that if i don't touch it
    // it will only get dropped when main quits, but compiler can't be of any
    // help in here to help ensure it for me. maybe use pin, idk
    let proxy = event_loop.create_proxy();

    // println!("pointer to config in main is: {}", (Rc::as_ptr(&config)) as usize);
    // println!("pointer to Rc<Config> in main is: {}", (&config as *const Rc<Config>) as usize);

    // to work properly with winit (at least on mac) status bar item needs
    // to be created after the event loop but before any windows are created
    add_status_bar_button(&config, &proxy);
    
    let mut window: Option<Window> = None;

    event_loop.run(move |event, event_loop, control_flow| {
        let now = Local::now();
        // println!("{}: {:?}. {:?}", now.format("%T %3f"), &event, &control_flow);

        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event: window_event, window_id: _, } => {
                match window_event {
                    WindowEvent::CloseRequested => {
                        println!("{}: {:?}. {:?}", now.format("%T %3f"), &window_event, &control_flow);
                        *control_flow = ControlFlow::Exit;
                        println!("set control flow to exit. control flow now: {:?}", &control_flow);
                    },
                    // destroy window when it loses focus
                    WindowEvent::Focused(is_focused) if !is_focused => {
                        window = None;
                    },
                    _ => (),
                }
            },
            Event::UserEvent(custom_event) => {
                println!("{}: {:?}. {:?}", now.format("%T %3f"), &custom_event, &control_flow);
                match custom_event {
                    CustomEvent::ToggleWindow (window_position) => {
                        let window_size = LogicalSize::<f64> {
                            width: config.window_width, height: config.window_height,
                        };

                        if window.is_none() {
                            let mut builder = WindowBuilder::new()
                                .with_decorations(false)
                                .with_inner_size(window_size);
                            if let Some(position) = window_position {
                                builder = builder.with_position(position);
                            }
                            let new_window = builder.build(event_loop).unwrap();
                            new_window.focus_window();
                            window = Some(new_window);
                        } else {
                            window = None;
                        }
                    }
                    CustomEvent::Exit => {
                        println!("{}: {:?}. {:?}", now.format("%T %3f"), &custom_event, &control_flow);
                        *control_flow = ControlFlow::Exit;
                        println!("set control flow to exit. control flow now: {:?}", &control_flow);
                    },
                    CustomEvent::GrayscaleToggle(is_grayscale) => {
                        was_grayscale = is_grayscale;
                    },
                }
            },
            Event::LoopDestroyed => {
                println!("{}: {:?}. {:?}", now.format("%T %3f"), &event, &control_flow);
                set_grayscale(was_grayscale);
            }
            _ => (),
        }
    });

    #[allow(unreachable_code)]
    Ok(())
}
