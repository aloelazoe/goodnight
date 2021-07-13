use std::{rc::Rc};
use cocoa::{
    appkit::{
        NSButton, NSStatusBar, NSStatusItem,
        NSVariableStatusItemLength,
    },
    base::{id, nil},
    foundation::{NSString, NSRect, NSPoint, NSSize},
};
use objc::{
    class, declare::ClassDecl, msg_send,
    runtime::{Object, Sel},
    sel, sel_impl
};
use winit::{
    event_loop::EventLoopProxy,
    dpi::LogicalPosition,
};
use crate::{
    config::Config,
    CustomEvent,
};

/// macos specific
pub fn add_status_bar_button(config: &Rc<Config>, loop_proxy: &EventLoopProxy<CustomEvent>) {
    // create a status bar item
    let tray_button = unsafe {
        let status_bar: *mut Object = msg_send![class!(NSStatusBar), systemStatusBar];
        let item = status_bar.statusItemWithLength_(NSVariableStatusItemLength);
        let title = NSString::alloc(nil).init_str(config.title.as_str());
        let button = item.button();
        NSButton::setTitle_(button, title);
        button
    };

    // define method selector
    let action_selector = sel!(onButtonAction:);
    
    // define action target class
    let superclass = class!(NSObject);
    let mut decl = ClassDecl::new("TrayActionTarget", superclass).unwrap();
    decl.add_ivar::<usize>("_config");
    decl.add_ivar::<usize>("_loop_proxy");
    unsafe {
        decl.add_method(
            action_selector,
            button_action as extern fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(init:loop_proxy:),
            init_button_target as extern fn(&mut Object, Sel, usize, usize) -> id,
        )
    }
    let action_target_class = decl.register();

    // instantiate action target class
    let action_target: id = unsafe {
        let action_target_empty: id = msg_send![action_target_class, alloc];
        msg_send![
            action_target_empty,
            init:pack_ptr_from_rc(&config) loop_proxy:pack_ptr(loop_proxy)
        ]
    };

    unsafe {
        tray_button.setTarget_(action_target);
        tray_button.setAction_(action_selector);
    }
}

fn pack_ptr<T>(reference: &T) -> usize {
    let pointer = reference as *const T;
    pointer as usize
}

fn pack_ptr_from_rc<T>(rc: &Rc<T>) -> usize {
    let pointer = Rc::as_ptr(rc);
    pointer as usize
}

unsafe fn unpack_ptr<T>(pointer_value: usize) -> &'static T {
    let pointer = pointer_value as *const T;
    &*pointer
}

extern fn button_action(this: &Object, _cmd: Sel, sender: id) {
    unsafe {
        let config: &Config = unpack_ptr(*this.get_ivar("_config"));
        let loop_proxy: &EventLoopProxy<CustomEvent> = unpack_ptr(
            *this.get_ivar("_loop_proxy")
        );

        let bounds: NSRect = msg_send![sender, bounds];
        let window: *const Object = msg_send![sender, window];
        let screen: *const Object = msg_send![window, screen];
        let screen_frame: NSRect = msg_send![screen, frame];
        let NSRect {size: NSSize { width: display_width, ..}, ..} = screen_frame;

        let bounds_in_window: NSRect = msg_send![sender, convertRect:bounds toView:nil];
        let bounds_on_screen: NSRect = msg_send![window, convertRectToScreen:bounds_in_window];
        let NSRect {
            origin: NSPoint {x, ..},
            size: NSSize {width, height}
        } = bounds_on_screen;

        let window_x = if x + config.window_width > display_width {
            x + width - config.window_width
        } else {
            x
        };
        let window_y = height;

        loop_proxy.send_event(CustomEvent::ToggleWindow(Some(LogicalPosition {
            x: window_x, y: window_y,
        }))).unwrap();
    }
}
extern fn init_button_target(this: &mut Object, _cmd: Sel, config: usize, loop_proxy: usize) -> id {
    unsafe {
        this.set_ivar("_config", config);
        this.set_ivar("_loop_proxy", loop_proxy);
        this as *mut Object
    }
}
