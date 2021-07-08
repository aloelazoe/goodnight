use std::{
    path::PathBuf,
    process::Command,
    rc::Rc,
};
// use cocoa::base::id;
// use objc::runtime::{Object, Sel};
use tray_item::TrayItem;
use winit::window::Window;

use crate::config::Config;
use crate::grayscale::{is_grayscale, set_grayscale};

#[allow(unused_variables)]
pub fn create_tray(config_path: PathBuf, config: &Config, was_grayscale: bool, window: Rc<Window>) {
    // 😴🌚☾☀︎
    let mut tray = TrayItem::new(&config.title, "").unwrap();
    tray.add_label(&format!("✨GRAY SCREEN FOR GAY BABES {}✨", &config.nighttime)).unwrap();
    #[cfg(debug_assertions)]
    tray.add_label(&format!("debug mode")).unwrap();

    tray.add_menu_item("edit settings - needs restart", move || {
        Command::new("open")
            .arg(&config_path)
            .output()
            .expect("failed to open config file in system default application");
    }).unwrap();

    tray.add_menu_item("show window", move || {
        window.set_visible(true);
    }).unwrap();

    let inner = tray.inner_mut();    
    // revert to the original grayscale setting when quitting the app
    // extern fn on_app_should_terminate(this: &mut Object, _cmd: Sel, _notification: id) {
    //     let was_grayscale: bool = unsafe { *(this.get_ivar("was_grayscale")) };
    //     set_grayscale(was_grayscale);
    // }
    // let delegate = unsafe {
    //     delegate!("AppDelegate", {
    //         was_grayscale: bool = was_grayscale,
    //         (applicationWillTerminate:) => on_app_should_terminate as extern fn(&mut Object, Sel, id)
    //     })
    // };
    // winit creates it's own delegate, it seems i would need to
    // extend it instead of creating another one
    // unsafe {inner.set_app_delegate(delegate);};

    // create a mutable reference from the raw pointer for capturing with closure
    // (raw pointers can't implement sync and send)
    // let delegate_ref = unsafe {&mut*delegate};
    inner.add_menu_item("toggle grayscale", move || {
        let should_be_set_to_grayscale = !is_grayscale();
        set_grayscale(should_be_set_to_grayscale);
        // keep track of manual toggles to avoid overriding them with initial value when quitting
        // unsafe {delegate_ref.set_ivar::<bool>("was_grayscale", should_be_set_to_grayscale)};
    }).unwrap();
    
    inner.add_quit_item("quit");

    inner.display();
}
