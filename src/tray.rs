use cocoa::{
    appkit::{NSButton, NSStatusBar, NSStatusItem, NSVariableStatusItemLength},
    base::{id, nil},
    foundation::NSString
};
use objc::{
    class, declare::ClassDecl, msg_send,
    runtime::{Object, Sel},
    sel, sel_impl
};
use winit::event_loop::EventLoopProxy;
use crate::CustomEvent;

pub fn add_status_bar_button(title: &str, loop_proxy: &EventLoopProxy<CustomEvent>) {
    // create a status bar item
    let tray_button = unsafe {
        let status_bar: *mut Object = msg_send![class!(NSStatusBar), systemStatusBar];
        let item = status_bar.statusItemWithLength_(NSVariableStatusItemLength);
        let title = NSString::alloc(nil).init_str(title);
        let button = item.button();
        NSButton::setTitle_(button, title);
        button
    };

    // define method selector
    let action_selector = sel!(onButtonAction:);
    
    // define action target class
    let superclass = class!(NSObject);
    let mut decl = ClassDecl::new("TrayActionTarget", superclass).unwrap();
    decl.add_ivar::<usize>("_loop_proxy");
    extern fn button_action(this: &Object, _cmd: Sel, sender: id) {
        unsafe {
            println!("button_action");
            dbg!(&*sender);

            let loop_proxy_ptr: usize = *this.get_ivar("_loop_proxy");
            let loop_proxy_ptr = loop_proxy_ptr as *const EventLoopProxy<CustomEvent>;
            let loop_proxy = &*loop_proxy_ptr;
            loop_proxy.send_event(CustomEvent::TrayButtonClick).unwrap();
        }
    }
    extern fn init(this: &mut Object, _cmd: Sel, loop_proxy: usize) -> id {
        unsafe {
            this.set_ivar("_loop_proxy", loop_proxy);
            this as *mut Object
        }
    }
    unsafe {
        decl.add_method(
            action_selector,
            button_action as extern fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(init:),
            init as extern fn(&mut Object, Sel, usize) -> id,
        )
    }
    let action_target_class = decl.register();
    
    let loop_proxy_ptr = loop_proxy as *const EventLoopProxy<CustomEvent>;
    let loop_proxy_ptr = loop_proxy_ptr as usize;

    // instantiate action target class
    let action_target: id = unsafe {
        let action_target_empty: id = msg_send![action_target_class, alloc];
        msg_send![action_target_empty, init:loop_proxy_ptr]
    };

    unsafe {
        dbg!(&*action_target);
    
        tray_button.setTarget_(action_target);
        tray_button.setAction_(action_selector);
    }
}
