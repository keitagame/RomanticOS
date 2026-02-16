pub mod vga;
pub mod keyboard;
pub mod timer;

pub fn init() {
    vga::init();
    keyboard::init();
    timer::init();
}
