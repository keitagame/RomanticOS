# RustOS Kernel - プロジェクト概要

## 📋 プロジェクト情報

**名前**: RustOS Kernel  
**バージョン**: 0.1.0  
**言語**: Rust (nightly)  
**アーキテクチャ**: x86_64  
**目的**: 教育用OSカーネルの完全実装

---

## ✨ 実装された機能

### 🧠 メモリ管理
- ✅ **ページング**: 4レベルページテーブル (PML4)
- ✅ **仮想メモリ**: カーネル/ユーザー空間分離
- ✅ **ヒープアロケータ**: 動的メモリ割り当て
- ✅ **フレームアロケータ**: 物理メモリ管理
- ✅ **メモリマッピング**: mmap/munmapシステムコール

### ⚙️ プロセス管理
- ✅ **プロセス構造**: PID、状態、コンテキスト
- ✅ **スケジューラ**: ラウンドロビン方式
- ✅ **コンテキストスイッチ**: 完全なレジスタ保存/復元
- ✅ **プロセスライフサイクル**: Ready → Running → Blocked → Terminated
- ✅ **マルチタスキング**: 複数プロセスの並行実行

### 📞 システムコール
- ✅ **I/O**: read, write
- ✅ **ファイル**: open, close
- ✅ **プロセス**: exit, fork (骨組み), execve (骨組み), getpid
- ✅ **メモリ**: mmap, munmap
- ✅ **その他**: sleep
- ✅ **統計**: システムコール呼び出し回数追跡

### 💾 ファイルシステム
- ✅ **VFS**: 仮想ファイルシステムレイヤー
- ✅ **Inode**: ファイル/ディレクトリメタデータ
- ✅ **階層構造**: ディレクトリツリー
- ✅ **ファイルディスクリプタ**: オープンファイル管理
- ✅ **基本操作**: create, mkdir, read, write, list

### 🔌 デバイスドライバ
- ✅ **VGA**: テキストモード 80x25、カラー出力
- ✅ **キーボード**: PS/2キーボード、スキャンコード処理
- ✅ **タイマー**: PIT、100Hz割り込み、アップタイム計測

### 🔧 低レベル機能
- ✅ **GDT**: Global Descriptor Table
- ✅ **IDT**: Interrupt Descriptor Table
- ✅ **TSS**: Task State Segment
- ✅ **割り込み処理**: 例外、ハードウェア割り込み
- ✅ **PIC**: 8259割り込みコントローラ

---

## 📊 統計情報

| 項目 | 値 |
|------|-----|
| 総コード行数 | ~2,500行 |
| Rustモジュール数 | 11 |
| 実装システムコール数 | 11 |
| サポートドライバ数 | 3 |
| ドキュメントページ数 | 4 |

---

## 🏗️ プロジェクト構造

```
rust-os-kernel/
├── src/
│   ├── main.rs              # カーネルエントリーポイント
│   ├── memory.rs            # メモリ管理 (~220行)
│   ├── process.rs           # プロセス管理 (~320行)
│   ├── syscall.rs           # システムコール (~330行)
│   ├── filesystem.rs        # ファイルシステム (~350行)
│   ├── gdt.rs              # GDT設定 (~70行)
│   ├── interrupts.rs        # 割り込み処理 (~140行)
│   ├── demo.rs             # デモプログラム (~200行)
│   └── drivers/
│       ├── mod.rs          # ドライバ初期化
│       ├── vga.rs          # VGAドライバ (~150行)
│       ├── keyboard.rs     # キーボードドライバ (~100行)
│       └── timer.rs        # タイマードライバ (~50行)
│
├── .cargo/
│   └── config.toml         # ビルド設定
│
├── Cargo.toml              # 依存関係
├── x86_64-unknown-none.json # ターゲット仕様
├── Makefile                # ビルド自動化
├── build.sh                # ビルドスクリプト
│
├── README.md               # プロジェクト説明
├── ARCHITECTURE.md         # アーキテクチャ詳細
├── DEVELOPMENT.md          # 開発者ガイド
└── .gitignore             # Git除外設定
```

---

## 🚀 クイックスタート

### 1. 環境セットアップ
```bash
# Rust nightlyをインストール
rustup default nightly

# 依存関係をチェック
./build.sh check
```

### 2. ビルド
```bash
# Makefileを使用
make build

# または直接cargoで
cargo build --release

# またはスクリプトで
./build.sh build
```

### 3. 実行
```bash
# QEMUで実行
make run

# または
cargo run --release

# または
./build.sh run
```

---

## 📖 ドキュメント

| ドキュメント | 内容 |
|-------------|------|
| [README.md](README.md) | プロジェクト概要、機能一覧 |
| [ARCHITECTURE.md](ARCHITECTURE.md) | 詳細なアーキテクチャ設計 |
| [DEVELOPMENT.md](DEVELOPMENT.md) | 開発者向けガイド |
| この文書 | プロジェクト全体の概要 |

---

## 🎯 主要な設計判断

### なぜRust?
- **メモリ安全性**: 所有権システムによりメモリリークやダングリングポインタを防止
- **ゼロコスト抽象化**: 高レベルコードでも高パフォーマンス
- **優れたツール**: Cargo、rustfmt、clippyなど

### なぜx86_64?
- **広くサポート**: QEMUや実機で容易にテスト可能
- **豊富な資料**: Intel SDM、OSDev Wikiなど
- **教育的価値**: 多くのOS教材がx86_64を使用

### なぜBootloader crate?
- **簡潔性**: 低レベルブートローダーの複雑さを隠蔽
- **標準的**: Rust OS開発で広く使用
- **保守性**: コミュニティによるメンテナンス

---

## 🔍 コードハイライト

### プロセススケジューラ
```rust
pub fn schedule(&mut self) -> Option<&mut Process> {
    // ラウンドロビン方式
    while let Some(pid) = self.ready_queue.pop_front() {
        if let Some(process) = self.processes.iter_mut()
            .find(|p| p.pid == pid && p.state == ProcessState::Ready) 
        {
            process.state = ProcessState::Running;
            return Some(process);
        }
    }
    None
}
```

### システムコールハンドラ
```rust
pub extern "C" fn syscall_handler(
    syscall_number: u64,
    arg1: u64, arg2: u64, arg3: u64,
    // ...
) -> i64 {
    match syscall_number {
        SYS_WRITE => sys_write(arg1 as i32, arg2 as *const u8, arg3 as usize),
        SYS_READ => sys_read(arg1 as i32, arg2 as *mut u8, arg3 as usize),
        // ...
    }
}
```

### メモリアロケーション
```rust
pub fn allocate_pages(count: usize) -> Option<VirtAddr> {
    for i in 0..count {
        let page = start_page + i as u64;
        let frame = frame_allocator.allocate_frame()?;
        let flags = Flags::PRESENT | Flags::WRITABLE | Flags::USER_ACCESSIBLE;
        
        mapper.map_to(page, frame, flags, &mut frame_allocator)?;
    }
    Some(start_page.start_address())
}
```

---

## 🧪 テストとデモ

### デモ機能
カーネルは起動時に以下のデモを実行します:

1. **メモリ管理デモ**: Vec、String、ページアロケーション
2. **ファイルシステムデモ**: ファイル作成、読み書き
3. **ドライバデモ**: VGAカラー、タイマー、キーボード
4. **システムコールデモ**: 統計情報表示
5. **プロセスデモ**: マルチタスキング

### 実行例
```
RustOS Kernel v0.1.0
Initializing...
[OK] GDT initialized
[OK] IDT initialized
[OK] Memory management initialized
...

=== Memory Management Demo ===
Allocating Vec...
Vec contents: [0, 1, 4, 9, 16, 25, 36, 49, 64, 81]

=== Filesystem Demo ===
Creating /test.txt...
Writing...
Reading...
File content: Hello from RustOS!
```

---

## 📈 今後の拡張計画

### 短期
- [ ] シェル実装
- [ ] ELFバイナリローダー
- [ ] より多くのドライバ (UART、RTC)

### 中期
- [ ] ext2ファイルシステム
- [ ] ATAディスクドライバ
- [ ] より高度なスケジューラ (CFS)

### 長期
- [ ] マルチプロセッササポート (SMP)
- [ ] ネットワークスタック (TCP/IP)
- [ ] グラフィカルユーザーインターフェース

---

## 🤝 コントリビューション

このプロジェクトへの貢献を歓迎します！

1. このリポジトリをFork
2. Feature branchを作成
3. 変更をCommit
4. BranchにPush
5. Pull Requestを作成

---

## 📚 学習リソース

- **Writing an OS in Rust**: https://os.phil-opp.com/
- **OSDev Wiki**: https://wiki.osdev.org/
- **Rust Book**: https://doc.rust-lang.org/book/
- **Intel SDM**: https://www.intel.com/sdm

---

## 📄 ライセンス

MIT License - 自由に使用、変更、配布できます

---

## 👨‍💻 作成者

RustOS Kernel開発チーム

**連絡先**: [プロジェクトリポジトリ]

---

## 🙏 謝辞

- Philipp Oppermann氏の "Writing an OS in Rust" チュートリアル
- Rust OSコミュニティ
- OSDev.orgコミュニティ
