# 開発者向けガイド

## カーネル開発入門

このガイドでは、RustOSカーネルの拡張方法について説明します。

## 目次
1. [環境セットアップ](#環境セットアップ)
2. [新しいシステムコールの追加](#新しいシステムコールの追加)
3. [新しいドライバの追加](#新しいドライバの追加)
4. [プロセススケジューラの改良](#プロセススケジューラの改良)
5. [デバッグ方法](#デバッグ方法)
6. [テスト](#テスト)

---

## 環境セットアップ

### 必要なツール

```bash
# Rust nightlyツールチェイン
rustup default nightly

# ソースコード
rustup component add rust-src --toolchain nightly

# LLVM tools
rustup component add llvm-tools-preview

# bootimage
cargo install bootimage

# QEMU (Ubuntu/Debian)
sudo apt install qemu-system-x86

# QEMU (macOS)
brew install qemu
```

### プロジェクトのクローン

```bash
git clone <repository>
cd rust-os-kernel
./build.sh check
```

---

## 新しいシステムコールの追加

### 手順

#### 1. システムコール番号を定義 (`src/syscall.rs`)

```rust
pub const SYS_MY_SYSCALL: u64 = 100;
```

#### 2. システムコールハンドラに追加

```rust
pub extern "C" fn syscall_handler(
    syscall_number: u64,
    // ... 引数
) -> i64 {
    match syscall_number {
        // ... 既存のケース
        SYS_MY_SYSCALL => sys_my_syscall(arg1, arg2),
        _ => -1,
    }
}
```

#### 3. 実装を追加

```rust
fn sys_my_syscall(arg1: u64, arg2: u64) -> i64 {
    // システムコールの実装
    crate::println!("My syscall called with {} {}", arg1, arg2);
    0 // 成功
}
```

#### 4. ユーザー空間ラッパーを追加 (`syscall::user`)

```rust
pub mod user {
    #[inline(always)]
    pub fn my_syscall(arg1: u64, arg2: u64) -> isize {
        unsafe {
            syscall2(SYS_MY_SYSCALL, arg1, arg2) as isize
        }
    }
}
```

### 使用例

```rust
use crate::syscall::user::my_syscall;

fn test() {
    let result = my_syscall(42, 100);
    println!("Result: {}", result);
}
```

---

## 新しいドライバの追加

### 手順

#### 1. ドライバモジュールを作成 (`src/drivers/mydevice.rs`)

```rust
use spin::Mutex;

static DEVICE: Mutex<Option<MyDevice>> = Mutex::new(None);

pub struct MyDevice {
    // デバイスの状態
    base_addr: u16,
}

impl MyDevice {
    pub fn new(base_addr: u16) -> Self {
        Self { base_addr }
    }
    
    pub fn read(&self) -> u8 {
        use x86_64::instructions::port::Port;
        let mut port = Port::new(self.base_addr);
        unsafe { port.read() }
    }
    
    pub fn write(&mut self, data: u8) {
        use x86_64::instructions::port::Port;
        let mut port = Port::new(self.base_addr);
        unsafe { port.write(data); }
    }
}

pub fn init() {
    let device = MyDevice::new(0x3F8); // I/Oポートアドレス
    *DEVICE.lock() = Some(device);
    crate::println!("MyDevice initialized");
}

pub fn handle_interrupt() {
    if let Some(device) = DEVICE.lock().as_ref() {
        let data = device.read();
        // 割り込み処理
    }
}
```

#### 2. ドライバを登録 (`src/drivers/mod.rs`)

```rust
pub mod mydevice;

pub fn init() {
    // ... 既存のドライバ
    mydevice::init();
}
```

#### 3. 割り込みハンドラを追加 (`src/interrupts.rs`)

```rust
idt[InterruptIndex::MyDevice.as_usize()]
    .set_handler_fn(mydevice_interrupt_handler);

extern "x86-interrupt" fn mydevice_interrupt_handler(
    _stack_frame: InterruptStackFrame
) {
    crate::drivers::mydevice::handle_interrupt();
    
    unsafe {
        Port::<u8>::new(0x20).write(0x20); // EOI
    }
}
```

---

## プロセススケジューラの改良

### 優先度ベーススケジューラの実装例

```rust
impl ProcessManager {
    pub fn schedule_priority(&mut self) -> Option<&mut Process> {
        // 優先度が最も高いプロセスを選択
        let mut highest_priority = 0;
        let mut selected_pid = None;
        
        for pid in &self.ready_queue {
            if let Some(process) = self.processes.iter()
                .find(|p| p.pid == *pid && p.state == ProcessState::Ready) 
            {
                if process.priority > highest_priority {
                    highest_priority = process.priority;
                    selected_pid = Some(*pid);
                }
            }
        }
        
        if let Some(pid) = selected_pid {
            // キューから削除
            self.ready_queue.retain(|&p| p != pid);
            
            let process = self.processes.iter_mut()
                .find(|p| p.pid == pid)?;
            process.state = ProcessState::Running;
            self.current_pid = Some(pid);
            Some(process)
        } else {
            None
        }
    }
}
```

### Completely Fair Scheduler (CFS) 風の実装

```rust
use alloc::collections::BTreeMap;

pub struct CFSScheduler {
    vruntime: BTreeMap<usize, u64>, // PID -> virtual runtime
}

impl CFSScheduler {
    pub fn schedule(&mut self, processes: &mut [Process]) -> Option<usize> {
        // 最小vランタイムのプロセスを選択
        self.vruntime.iter()
            .min_by_key(|(_, vrt)| *vrt)
            .map(|(pid, _)| *pid)
    }
    
    pub fn update_vruntime(&mut self, pid: usize, real_time: u64) {
        let vrt = self.vruntime.entry(pid).or_insert(0);
        *vrt += real_time; // 優先度による重み付けも可能
    }
}
```

---

## デバッグ方法

### シリアルポート出力

```rust
// src/drivers/serial.rs を追加
use uart_16550::SerialPort;
use spin::Mutex;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::drivers::serial::_print(format_args!($($arg)*));
    };
}
```

### QEMU with GDB

```bash
# QEMUを起動 (GDBサーバー有効)
qemu-system-x86_64 -drive format=raw,file=target/x86_64-unknown-none/debug/bootimage-rust-os-kernel.bin -s -S

# 別のターミナルでGDB起動
gdb target/x86_64-unknown-none/debug/rust-os-kernel
(gdb) target remote :1234
(gdb) break kernel_main
(gdb) continue
```

### カーネルパニック情報

```rust
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("KERNEL PANIC!");
    println!("Location: {:?}", info.location());
    println!("Message: {:?}", info.message());
    
    // スタックトレース (unwindが有効な場合)
    #[cfg(feature = "backtrace")]
    print_backtrace();
    
    loop {
        x86_64::instructions::hlt();
    }
}
```

---

## テスト

### 単体テスト

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_memory_allocation() {
        let v = alloc::vec![1, 2, 3];
        assert_eq!(v.len(), 3);
    }
}
```

### 統合テスト

```bash
# tests/integration_test.rs を作成
cargo test
```

### QEMUでのテスト実行

```bash
# 自動終了機能を使用
cargo test --target x86_64-unknown-none
```

---

## パフォーマンスプロファイリング

### タイマーを使った計測

```rust
use crate::drivers::timer;

fn benchmark_function() {
    let start = timer::get_ticks();
    
    // 計測したい処理
    expensive_operation();
    
    let end = timer::get_ticks();
    println!("Time: {} ticks", end - start);
}
```

### メモリ使用量の追跡

```rust
// グローバルアロケータのラッパー
struct ProfilingAllocator;

unsafe impl GlobalAlloc for ProfilingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATED_BYTES.fetch_add(layout.size(), Ordering::SeqCst);
        // 実際のアロケーション
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        ALLOCATED_BYTES.fetch_sub(layout.size(), Ordering::SeqCst);
        // 実際のデアロケーション
    }
}
```

---

## コーディング規約

### Rust標準スタイル

```bash
cargo fmt
cargo clippy
```

### カーネル固有のルール

1. **`unsafe`の使用**
   - 必要最小限に抑える
   - SAFETYコメントを追加

```rust
// SAFETY: This is safe because...
unsafe {
    // unsafe code
}
```

2. **エラーハンドリング**
   - `Result`を使用
   - パニックは最後の手段

```rust
pub fn do_something() -> Result<(), &'static str> {
    if condition {
        Ok(())
    } else {
        Err("Error message")
    }
}
```

3. **ドキュメント**
   - 公開APIには必ずドキュメントコメント

```rust
/// Does something important
///
/// # Arguments
/// * `arg1` - First argument
///
/// # Returns
/// Result indicating success or failure
pub fn important_function(arg1: u64) -> Result<(), Error> {
    // ...
}
```

---

## コントリビューション

1. Forkする
2. Feature branchを作成 (`git checkout -b feature/amazing-feature`)
3. Commit (`git commit -m 'Add amazing feature'`)
4. Push (`git push origin feature/amazing-feature`)
5. Pull Requestを開く

---

## 参考資料

- [Rust OS Dev Tutorial](https://os.phil-opp.com/)
- [OSDev Wiki](https://wiki.osdev.org/)
- [x86_64 crate docs](https://docs.rs/x86_64/)
- [Intel Software Developer Manual](https://www.intel.com/sdm)
