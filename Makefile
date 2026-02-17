.PHONY: all build run clean test check help

# デフォルトターゲット
all: build

# ビルド
build:
	@echo "Building RustOS Kernel..."
	@cargo build --release

# 実行
run:
	@echo "Running RustOS Kernel in QEMU..."
	@cargo run --release

# デバッグビルド
debug:
	@echo "Building debug version..."
	@cargo build

# デバッグ実行
run-debug:
	@echo "Running debug version in QEMU..."
	@cargo run

# QEMUをGDBサーバーモードで起動
debug-gdb:
	@echo "Starting QEMU with GDB server..."
	@qemu-system-x86_64 \
		-drive format=raw,file=target/x86_64-unknown-none/debug/rust-os-kernel \
		-serial stdio \
		-s -S

# クリーン
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean

# テスト
test:
	@echo "Running tests..."
	@cargo test

# 依存関係チェック
check:
	@echo "Checking dependencies..."
	@./build.sh check

# フォーマット
fmt:
	@echo "Formatting code..."
	@cargo fmt

# Linting
clippy:
	@echo "Running clippy..."
	@cargo clippy

# ドキュメント生成
doc:
	@echo "Generating documentation..."
	@cargo doc --no-deps --open

# ヘルプ
help:
	@echo "RustOS Kernel Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  all        - Build the kernel (default)"
	@echo "  build      - Build release version"
	@echo "  run        - Build and run in QEMU"
	@echo "  debug      - Build debug version"
	@echo "  run-debug  - Run debug version"
	@echo "  debug-gdb  - Start QEMU with GDB server"
	@echo "  clean      - Remove build artifacts"
	@echo "  test       - Run tests"
	@echo "  check      - Check dependencies"
	@echo "  fmt        - Format code"
	@echo "  clippy     - Run linter"
	@echo "  doc        - Generate and open documentation"
	@echo "  help       - Show this help message"

# カーネルサイズ表示
size: build
	@echo "Kernel size:"
	@ls -lh target/x86_64-unknown-none/release/rust-os-kernel

# 全チェック (CI用)
ci: fmt clippy test build
	@echo "All CI checks passed!"
