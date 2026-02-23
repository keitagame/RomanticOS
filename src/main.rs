
#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
mod boot;
extern crate alloc;


use core::panic::PanicInfo;

mod memory;
mod process;
mod syscall;
mod filesystem;
mod drivers;
mod interrupts;
mod gdt;
mod demo;


#[no_mangle]
pub extern "C" fn _start(multiboot_magic: u32, multiboot_info_addr: u32) -> ! {
    unsafe { let vga = 0xb8000 as *mut u8; *vga = b'H'; *vga.add(1) = 0x0f; }
    println!("RustOS Kernel v0.1.0");
    println!("Booted via GRUB (Multiboot2)");

    // 必要なら multiboot_info_addr をパースしてメモリマップを取得できる
    // まずは boot_info を使わずに固定初期化でOK

    kernel_main();
}


//entry_point!(kernel_main);

fn kernel_main() -> ! {
    println!("RustOS Kernel v0.1.0");
    println!("Initializing...");

    // GDT初期化
    gdt::init();
    println!("[OK] GDT initialized");

    // 割り込み初期化
    interrupts::init_idt();
    println!("[OK] IDT initialized");

    // メモリ管理初期化
    memory::init();
    println!("[OK] Memory management initialized");

    // ヒープアロケータ初期化
    memory::init_heap().expect("Heap initialization failed");
    println!("[OK] Heap allocator initialized");

    // プロセス管理初期化
    process::init();
    println!("[OK] Process manager initialized");

    // ファイルシステム初期化
    filesystem::init();
    println!("[OK] Filesystem initialized");

    // ドライバ初期化
    drivers::init();
    println!("[OK] Drivers initialized");

    // システムコール初期化
    syscall::init();
    println!("[OK] Syscall handler initialized");

    println!("\nKernel initialization complete!");
    println!("Starting init process...\n");

    // デモ実行
    demo::run_complete_demo();

    // initプロセス起動
    process::spawn_init_process();

    // スケジューラ開始
    process::scheduler::start();

    // ここには到達しないbo
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("KERNEL PANIC: {}", info);
    loop {
        x86_64::instructions::hlt();
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

// 簡易printlnマクロ
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::drivers::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
