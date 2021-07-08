#[link(name = "ApplicationServices", kind = "framework")]
extern {
    fn CGDisplayUsesForceToGray() -> bool;
    fn CGDisplayForceToGray(forceToGray: bool);
}

pub fn set_grayscale(on: bool) {
    unsafe {
        CGDisplayForceToGray(on);
    }
}

pub fn is_grayscale() -> bool {
    unsafe {
        CGDisplayUsesForceToGray()
    }
}
