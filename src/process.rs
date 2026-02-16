use alloc::collections::VecDeque;

use alloc::vec;
use alloc::vec::Vec;

use alloc::boxed::Box;
use spin::Mutex;
use x86_64::VirtAddr;
use core::sync::atomic::{AtomicUsize, Ordering};

static PID_COUNTER: AtomicUsize = AtomicUsize::new(1);
static PROCESS_MANAGER: Mutex<Option<ProcessManager>> = Mutex::new(None);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct ProcessContext {
    pub rsp: u64,
    pub rbp: u64,
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub rflags: u64,
}

impl Default for ProcessContext {
    fn default() -> Self {
        Self {
            rsp: 0,
            rbp: 0,
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: 0,
            rflags: 0x202, // IF (割り込み有効)
        }
    }
}

pub struct Process {
    pub pid: usize,
    pub state: ProcessState,
    pub context: ProcessContext,
    pub kernel_stack: Vec<u8>,
    pub user_stack: Option<VirtAddr>,
    pub page_table: Option<VirtAddr>,
    pub priority: u8,
    pub time_slice: usize,
}

impl Process {
    pub fn new(entry_point: u64) -> Self {
        let pid = PID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut kernel_stack = vec![0u8; 8192]; // 8KB カーネルスタック
        
        let mut context = ProcessContext::default();
        context.rip = entry_point;
        context.rsp = (kernel_stack.as_ptr() as u64) + 8192;
        context.rbp = context.rsp;

        Self {
            pid,
            state: ProcessState::Ready,
            context,
            kernel_stack,
            user_stack: None,
            page_table: None,
            priority: 10,
            time_slice: 10,
        }
    }

    pub fn with_user_stack(mut self, stack_addr: VirtAddr) -> Self {
        self.user_stack = Some(stack_addr);
        self.context.rsp = stack_addr.as_u64();
        self
    }
}

pub struct ProcessManager {
    processes: Vec<Process>,
    ready_queue: VecDeque<usize>,
    current_pid: Option<usize>,
    scheduler_ticks: usize,
}

impl ProcessManager {
    fn new() -> Self {
        Self {
            processes: Vec::new(),
            ready_queue: VecDeque::new(),
            current_pid: None,
            scheduler_ticks: 0,
        }
    }

    pub fn add_process(&mut self, process: Process) -> usize {
        let pid = process.pid;
        self.processes.push(process);
        self.ready_queue.push_back(pid);
        pid
    }

    pub fn get_current_process(&self) -> Option<&Process> {
        self.current_pid
            .and_then(|pid| self.processes.iter().find(|p| p.pid == pid))
    }

    pub fn get_current_process_mut(&mut self) -> Option<&mut Process> {
        self.current_pid
            .and_then(|pid| self.processes.iter_mut().find(|p| p.pid == pid))
    }

    pub fn schedule(&mut self) -> Option<&mut Process> {
        self.scheduler_ticks += 1;

        // 現在のプロセスをReadyに戻す
        if let Some(current) = self.get_current_process_mut() {
            if current.state == ProcessState::Running {
                current.state = ProcessState::Ready;
                self.ready_queue.push_back(current.pid);
            }
        }

        // 次のプロセスを選択
        while let Some(pid) = self.ready_queue.pop_front() {
            if let Some(process) = self.processes.iter_mut().find(|p| p.pid == pid) {
                if process.state == ProcessState::Ready {
                    process.state = ProcessState::Running;
                    self.current_pid = Some(pid);
                    return Some(process);
                }
            }
        }

        None
    }

    pub fn terminate_current(&mut self) {
        if let Some(pid) = self.current_pid {
            if let Some(process) = self.processes.iter_mut().find(|p| p.pid == pid) {
                process.state = ProcessState::Terminated;
            }
            self.current_pid = None;
        }
    }

    pub fn block_current(&mut self) {
        if let Some(process) = self.get_current_process_mut() {
            process.state = ProcessState::Blocked;
        }
        self.current_pid = None;
    }

    pub fn unblock_process(&mut self, pid: usize) {
        if let Some(process) = self.processes.iter_mut().find(|p| p.pid == pid) {
            if process.state == ProcessState::Blocked {
                process.state = ProcessState::Ready;
                self.ready_queue.push_back(pid);
            }
        }
    }
}

pub fn init() {
    *PROCESS_MANAGER.lock() = Some(ProcessManager::new());
}

pub fn spawn_process(entry_point: u64) -> usize {
    let mut manager = PROCESS_MANAGER.lock();
    if let Some(manager) = manager.as_mut() {
        // ユーザースタック割り当て
        let stack_addr = crate::memory::allocate_pages(4) // 16KB
            .expect("Failed to allocate user stack");
        
        let process = Process::new(entry_point)
            .with_user_stack(stack_addr + 0x4000); // スタックトップ
        
        manager.add_process(process)
    } else {
        panic!("Process manager not initialized");
    }
}

pub fn spawn_init_process() {
    // initプロセスのエントリーポイント
    extern "C" fn init_process() {
        crate::println!("Init process started (PID: 1)");
        
        // いくつかのテストプロセスを起動
        spawn_process(test_process_1 as u64);
        spawn_process(test_process_2 as u64);
        
        loop {
            // initプロセスは基本的に待機
            x86_64::instructions::hlt();
        }
    }

    spawn_process(init_process as u64);
}

extern "C" fn test_process_1() {
    for i in 0..5 {
        crate::println!("Process 1: iteration {}", i);
        for _ in 0..100000 { unsafe { core::arch::asm!("nop"); } }
    }
    exit(0);
}

extern "C" fn test_process_2() {
    for i in 0..5 {
        crate::println!("Process 2: iteration {}", i);
        for _ in 0..100000 { unsafe { core::arch::asm!("nop"); } }
    }
    exit(0);
}

pub fn exit(code: i32) {
    let mut manager = PROCESS_MANAGER.lock();
    if let Some(manager) = manager.as_mut() {
        manager.terminate_current();
    }
}

pub mod scheduler {
    use super::*;

    pub fn start() -> ! {
        loop {
            tick();
            x86_64::instructions::hlt();
        }
    }

    pub fn tick() {
        let mut manager = PROCESS_MANAGER.lock();
        if let Some(manager) = manager.as_mut() {
            if let Some(_next_process) = manager.schedule() {
                // コンテキストスイッチ実行
                // 実際の実装ではアセンブリでレジスタを保存/復元
            }
        }
    }
}

// コンテキストスイッチ用のアセンブリ関数
#[unsafe(naked)]
pub unsafe extern "C" fn switch_context(
    old_context: *mut ProcessContext,
    new_context: *const ProcessContext,
) {
    core::arch::naked_asm!(
        // 現在のコンテキストを保存
        "mov [rdi + 0x00], rsp",
        "mov [rdi + 0x08], rbp",
        "mov [rdi + 0x10], rax",
        "mov [rdi + 0x18], rbx",
        "mov [rdi + 0x20], rcx",
        "mov [rdi + 0x28], rdx",
        "mov [rdi + 0x30], rsi",
        "mov [rdi + 0x38], rdi",
        "mov [rdi + 0x40], r8",
        "mov [rdi + 0x48], r9",
        "mov [rdi + 0x50], r10",
        "mov [rdi + 0x58], r11",
        "mov [rdi + 0x60], r12",
        "mov [rdi + 0x68], r13",
        "mov [rdi + 0x70], r14",
        "mov [rdi + 0x78], r15",
        
        // 新しいコンテキストを復元
        "mov rsp, [rsi + 0x00]",
        "mov rbp, [rsi + 0x08]",
        "mov rax, [rsi + 0x10]",
        "mov rbx, [rsi + 0x18]",
        "mov rcx, [rsi + 0x20]",
        "mov rdx, [rsi + 0x28]",
        "mov r8,  [rsi + 0x40]",
        "mov r9,  [rsi + 0x48]",
        "mov r10, [rsi + 0x50]",
        "mov r11, [rsi + 0x58]",
        "mov r12, [rsi + 0x60]",
        "mov r13, [rsi + 0x68]",
        "mov r14, [rsi + 0x70]",
        "mov r15, [rsi + 0x78]",
        "mov rdi, [rsi + 0x38]",
        "mov rsi, [rsi + 0x30]",
        
        "ret",
        //options(noreturn)
    );
}
