use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use lazy_static::lazy_static;
use crate::gdt;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        // 例外ハンドラ
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        
        // ハードウェア割り込み
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);
        
        // システムコール (int 0x80)
        idt[0x80].set_handler_fn(syscall_interrupt_handler);
        
        idt
    };
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = 32,
    Keyboard = 33,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub fn init_idt() {
    IDT.load();
    init_pics();
}

fn init_pics() {
    use pic8259::ChainedPics;
    use spin::Mutex;

    const PIC_1_OFFSET: u8 = 32;
    const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

    static PICS: Mutex<ChainedPics> =
        Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

    unsafe {
        PICS.lock().initialize();
    }
    x86_64::instructions::interrupts::enable();
}

// 例外ハンドラ

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    crate::println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    crate::println!("EXCEPTION: PAGE FAULT");
    crate::println!("Accessed Address: {:?}", Cr2::read());
    crate::println!("Error Code: {:?}", error_code);
    crate::println!("{:#?}", stack_frame);
    
    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    crate::println!("EXCEPTION: GENERAL PROTECTION FAULT");
    crate::println!("Error Code: {:#x}", error_code);
    crate::println!("{:#?}", stack_frame);
    
    loop {
        x86_64::instructions::hlt();
    }
}

// ハードウェア割り込みハンドラ

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    crate::drivers::timer::handle_interrupt();
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    crate::drivers::keyboard::handle_interrupt();
}

// システムコール割り込みハンドラ
extern "x86-interrupt" fn syscall_interrupt_handler(mut stack_frame: InterruptStackFrame) {
    // レジスタからシステムコール番号と引数を取得
    // 注: 実際の実装ではスタックフレームからレジスタ値を取得
    // この簡易版では、システムコールハンドラを直接呼び出すことはできない
    
    // システムコールの戻り値をraxに設定
    // stack_frame に戻り値を設定する処理
    
    crate::println!("Syscall interrupt received");
}

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
