
pub mod vga;
pub mod keyboard;
pub mod timer;

use spin::Once;

static DRIVERS_INITIALIZED: Once<bool> = Once::new();

pub fn init() {
    DRIVERS_INITIALIZED.call_once(|| {
        vga::init();
        keyboard::init();
        timer::init();
        true
    });
}