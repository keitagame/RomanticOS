use x86_64::structures::idt::InterruptStackFrame;
use spin::Mutex;

// システムコール番号
pub const SYS_READ: u64 = 0;
pub const SYS_WRITE: u64 = 1;
pub const SYS_OPEN: u64 = 2;
pub const SYS_CLOSE: u64 = 3;
pub const SYS_EXIT: u64 = 60;
pub const SYS_FORK: u64 = 57;
pub const SYS_EXECVE: u64 = 59;
pub const SYS_GETPID: u64 = 39;
pub const SYS_SLEEP: u64 = 35;
pub const SYS_MMAP: u64 = 9;
pub const SYS_MUNMAP: u64 = 11;

static SYSCALL_STATS: Mutex<SyscallStats> = Mutex::new(SyscallStats::new());

struct SyscallStats {
    total_calls: u64,
    calls_by_type: [u64; 256],
}

impl SyscallStats {
    const fn new() -> Self {
        Self {
            total_calls: 0,
            calls_by_type: [0; 256],
        }
    }
}

pub fn init() {
    // システムコール用の割り込みを設定
    // x86_64では通常 int 0x80 またはsyscall命令を使用
    crate::println!("Syscall handler registered");
}

/// システムコールハンドラ
/// レジスタマッピング:
/// rax: システムコール番号
/// rdi: 引数1
/// rsi: 引数2
/// rdx: 引数3
/// r10: 引数4
/// r8:  引数5
/// r9:  引数6
/// 戻り値: rax
#[no_mangle]
pub extern "C" fn syscall_handler(
    syscall_number: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
    arg6: u64,
) -> i64 {
    // 統計を更新
    {
        let mut stats = SYSCALL_STATS.lock();
        stats.total_calls += 1;
        if (syscall_number as usize) < 256 {
            stats.calls_by_type[syscall_number as usize] += 1;
        }
    }

    let result = match syscall_number {
        SYS_READ => sys_read(arg1 as i32, arg2 as *mut u8, arg3 as usize),
        SYS_WRITE => sys_write(arg1 as i32, arg2 as *const u8, arg3 as usize),
        SYS_OPEN => sys_open(arg1 as *const u8, arg2 as i32, arg3 as u32),
        SYS_CLOSE => sys_close(arg1 as i32),
        SYS_EXIT => sys_exit(arg1 as i32),
        SYS_FORK => sys_fork(),
        SYS_EXECVE => sys_execve(arg1 as *const u8, arg2 as *const *const u8, arg3 as *const *const u8),
        SYS_GETPID => sys_getpid(),
        SYS_SLEEP => sys_sleep(arg1),
        SYS_MMAP => sys_mmap(arg1 as u64, arg2 as usize, arg3 as i32, arg4 as i32, arg5 as i32, arg6 as i64),
        SYS_MUNMAP => sys_munmap(arg1 as u64, arg2 as usize),
        _ => {
            crate::println!("Unknown syscall: {}", syscall_number);
            -1 // ENOSYS
        }
    };

    result
}

// システムコール実装

fn sys_read(fd: i32, buf: *mut u8, count: usize) -> i64 {
    if fd < 0 || buf.is_null() {
        return -1; // EINVAL
    }

    match fd {
        0 => { // stdin
            // キーボード入力から読み込み
            let read = crate::drivers::keyboard::read_bytes(
                unsafe { core::slice::from_raw_parts_mut(buf, count) }
            );
            read as i64
        }
        _ => {
            // ファイルシステムから読み込み
            crate::filesystem::read(fd, unsafe { core::slice::from_raw_parts_mut(buf, count) })
        }
    }
}

fn sys_write(fd: i32, buf: *const u8, count: usize) -> i64 {
    if fd < 0 || buf.is_null() {
        return -1; // EINVAL
    }

    match fd {
        1 | 2 => { // stdout, stderr
            let slice = unsafe { core::slice::from_raw_parts(buf, count) };
            if let Ok(s) = core::str::from_utf8(slice) {
                crate::print!("{}", s);
                count as i64
            } else {
                -1
            }
        }
        _ => {
            // ファイルシステムへ書き込み
            crate::filesystem::write(fd, unsafe { core::slice::from_raw_parts(buf, count) })
        }
    }
}

fn sys_open(pathname: *const u8, flags: i32, mode: u32) -> i64 {
    if pathname.is_null() {
        return -1; // EINVAL
    }

    // パス名を読み取る
    let path = unsafe {
        let mut len = 0;
        while len < 4096 && *pathname.add(len) != 0 {
            len += 1;
        }
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(pathname, len))
    };

    crate::filesystem::open(path, flags, mode)
}

fn sys_close(fd: i32) -> i64 {
    crate::filesystem::close(fd)
}

fn sys_exit(status: i32) -> i64 {
    crate::println!("Process exiting with status: {}", status);
    crate::process::exit(status);
    
    // プロセスを終了させるのでここには戻らない
    unreachable!()
}

fn sys_fork() -> i64 {
    // fork実装 - 現在のプロセスを複製
    crate::println!("fork() called - not fully implemented");
    -1 // ENOSYS - 簡略版では未実装
}

fn sys_execve(filename: *const u8, argv: *const *const u8, envp: *const *const u8) -> i64 {
    if filename.is_null() {
        return -1; // EINVAL
    }

    crate::println!("execve() called - not fully implemented");
    -1 // ENOSYS
}

fn sys_getpid() -> i64 {
    // 現在のプロセスIDを返す
    // 簡略版: 固定値を返す
    1
}

fn sys_sleep(nanoseconds: u64) -> i64 {
    // プロセスをスリープ
    crate::println!("sleep({}) called", nanoseconds);
    
    // 簡易実装: ビジーウェイト
    for _ in 0..nanoseconds / 1000 {
        unsafe { core::arch::asm!("pause"); }
    }
    
    0
}

fn sys_mmap(addr: u64, length: usize, prot: i32, flags: i32, fd: i32, offset: i64) -> i64 {
    // メモリマッピング
    let pages = (length + 4095) / 4096;
    
    if let Some(virt_addr) = crate::memory::allocate_pages(pages) {
        virt_addr.as_u64() as i64
    } else {
        -1 // ENOMEM
    }
}

fn sys_munmap(addr: u64, length: usize) -> i64 {
    // メモリマッピング解除
    let pages = (length + 4095) / 4096;
    crate::memory::deallocate_pages(x86_64::VirtAddr::new(addr), pages);
    0
}

// ユーザー空間から呼び出すためのラッパー関数（例）
pub mod user {
    use super::*;

    #[inline(always)]
    pub fn write(fd: i32, buf: &[u8]) -> isize {
        unsafe {
            syscall3(SYS_WRITE, fd as u64, buf.as_ptr() as u64, buf.len() as u64) as isize
        }
    }

    #[inline(always)]
    pub fn read(fd: i32, buf: &mut [u8]) -> isize {
        unsafe {
            syscall3(SYS_READ, fd as u64, buf.as_mut_ptr() as u64, buf.len() as u64) as isize
        }
    }

    #[inline(always)]
    pub fn exit(status: i32) -> ! {
        unsafe {
            syscall1(SYS_EXIT, status as u64);
        }
        unreachable!()
    }

    #[inline(always)]
    pub fn getpid() -> i32 {
        unsafe { syscall0(SYS_GETPID) as i32 }
    }

    // システムコールを発行するアセンブリラッパー
    #[inline(always)]
    unsafe fn syscall0(number: u64) -> i64 {
        let ret: i64;
        core::arch::asm!(
            "int 0x80",
            in("rax") number,
            lateout("rax") ret,
        );
        ret
    }

    #[inline(always)]
    unsafe fn syscall1(number: u64, arg1: u64) -> i64 {
        let ret: i64;
        core::arch::asm!(
            "int 0x80",
            in("rax") number,
            in("rdi") arg1,
            lateout("rax") ret,
        );
        ret
    }

    #[inline(always)]
    unsafe fn syscall3(number: u64, arg1: u64, arg2: u64, arg3: u64) -> i64 {
        let ret: i64;
        core::arch::asm!(
            "int 0x80",
            in("rax") number,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            lateout("rax") ret,
        );
        ret
    }
}

pub fn print_stats() {
    let stats = SYSCALL_STATS.lock();
    crate::println!("Syscall Statistics:");
    crate::println!("  Total calls: {}", stats.total_calls);
    crate::println!("  read():  {}", stats.calls_by_type[SYS_READ as usize]);
    crate::println!("  write(): {}", stats.calls_by_type[SYS_WRITE as usize]);
    crate::println!("  open():  {}", stats.calls_by_type[SYS_OPEN as usize]);
    crate::println!("  close(): {}", stats.calls_by_type[SYS_CLOSE as usize]);
}
