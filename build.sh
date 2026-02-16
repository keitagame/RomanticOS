#!/bin/bash

# RustOS Kernel ビルド・実行スクリプト

set -e

echo "==================================="
echo "  RustOS Kernel Build Script"
echo "==================================="

# カラー出力
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 環境チェック
check_dependencies() {
    echo -e "${YELLOW}Checking dependencies...${NC}"
    
    if ! command -v rustc &> /dev/null; then
        echo -e "${RED}Error: Rust is not installed${NC}"
        exit 1
    fi
    
    if ! rustc --version | grep -q "nightly"; then
        echo -e "${YELLOW}Warning: Not using nightly Rust${NC}"
        echo "Switching to nightly..."
        rustup default nightly
    fi
    
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}Error: Cargo is not installed${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✓ Rust toolchain OK${NC}"
}

# bootimageのインストール
install_bootimage() {
    if ! command -v bootimage &> /dev/null; then
        echo -e "${YELLOW}Installing bootimage...${NC}"
        cargo install bootimage
    fi
    echo -e "${GREEN}✓ bootimage OK${NC}"
}

# LLVMツールのインストール
install_llvm_tools() {
    echo -e "${YELLOW}Installing LLVM tools...${NC}"
    rustup component add llvm-tools-preview 2>/dev/null || true
    echo -e "${GREEN}✓ LLVM tools OK${NC}"
}

# ビルドターゲットの追加
add_target() {
    echo -e "${YELLOW}Adding build target...${NC}"
    rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu 2>/dev/null || true
    echo -e "${GREEN}✓ Build target OK${NC}"
}

# ビルド
build() {
    echo ""
    echo -e "${YELLOW}Building kernel...${NC}"
    cargo build --release
    echo -e "${GREEN}✓ Build complete${NC}"
}

# 実行
run() {
    if ! command -v qemu-system-x86_64 &> /dev/null; then
        echo -e "${RED}Error: QEMU is not installed${NC}"
        echo "Install QEMU with: sudo apt install qemu-system-x86"
        exit 1
    fi
    
    echo ""
    echo -e "${YELLOW}Starting kernel in QEMU...${NC}"
    echo "Press Ctrl+A then X to exit QEMU"
    echo ""
    
    cargo run --release
}

# クリーン
clean() {
    echo -e "${YELLOW}Cleaning build artifacts...${NC}"
    cargo clean
    echo -e "${GREEN}✓ Clean complete${NC}"
}

# テスト
test() {
    echo -e "${YELLOW}Running tests...${NC}"
    cargo test
    echo -e "${GREEN}✓ Tests complete${NC}"
}

# メイン処理
main() {
    case "${1:-build}" in
        check)
            check_dependencies
            install_bootimage
            install_llvm_tools
            add_target
            ;;
        build)
            check_dependencies
            install_bootimage
            install_llvm_tools
            add_target
            build
            ;;
        run)
            check_dependencies
            install_bootimage
            install_llvm_tools
            add_target
            build
            run
            ;;
        clean)
            clean
            ;;
        test)
            check_dependencies
            test
            ;;
        *)
            echo "Usage: $0 {check|build|run|clean|test}"
            echo ""
            echo "Commands:"
            echo "  check  - Check dependencies and install required tools"
            echo "  build  - Build the kernel (default)"
            echo "  run    - Build and run the kernel in QEMU"
            echo "  clean  - Clean build artifacts"
            echo "  test   - Run tests"
            exit 1
            ;;
    esac
}

main "$@"
