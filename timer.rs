use x86_64::instructions::port::Port;
use core::sync::atomic::{AtomicUsize, Ordering};

const PIT_FREQUENCY: usize = 1193182;
const TARGET_FREQUENCY: usize = 100; // 100Hz (10ms tick)

static TICKS: AtomicUsize = AtomicUsize::new(0);

pub fn init() {
    let divisor = PIT_FREQUENCY / TARGET_FREQUENCY;

    unsafe {
        // コマンドレジスタ: チャンネル0、ロー/ハイバイト、モード3
        Port::<u8>::new(0x43).write(0x36);

        // 分周値を設定
        Port::<u8>::new(0x40).write((divisor & 0xFF) as u8);
        Port::<u8>::new(0x40).write((divisor >> 8) as u8);
    }

    crate::println!("Timer initialized: {} Hz", TARGET_FREQUENCY);
}

pub fn handle_interrupt() {
    TICKS.fetch_add(1, Ordering::SeqCst);

    // スケジューラのティック処理
    crate::process::scheduler::tick();

    // 割り込みコントローラに通知
    unsafe {
        Port::<u8>::new(0x20).write(0x20);
    }
}

pub fn get_ticks() -> usize {
    TICKS.load(Ordering::SeqCst)
}

pub fn get_uptime_ms() -> usize {
    (get_ticks() * 1000) / TARGET_FREQUENCY
}

pub fn sleep_ms(ms: usize) {
    let target = get_uptime_ms() + ms;
    while get_uptime_ms() < target {
        x86_64::instructions::hlt();
    }
}
