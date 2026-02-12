#!/bin/bash

# aicmd 多平台编译脚本
# 支持 Windows, Linux, macOS (Intel/ARM)

set -e

VERSION=$(grep '^version' Cargo.toml | cut -d'"' -f2)
NAME="aicmd"
RELEASE_DIR="releases"

echo "=== aicmd v${VERSION} 多平台编译 ==="
echo ""

# 创建输出目录
mkdir -p ${RELEASE_DIR}

# 检查是否安装了 cross
if ! command -v cross &> /dev/null; then
    echo "正在安装 cross..."
    cargo install cross
fi

# 检查是否安装了 cargo-zigbuild (用于 macOS 交叉编译)
if ! command -v cargo-zigbuild &> /dev/null; then
    echo "提示: 建议安装 cargo-zigbuild 以支持更好的交叉编译"
    echo "      cargo install cargo-zigbuild"
    echo ""
fi

echo "开始编译..."
echo ""

# ==================== Linux ====================
echo "[1/6] Linux x86_64 (GNU)..."
cross build --release --target x86_64-unknown-linux-gnu 2>/dev/null || cargo build --release --target x86_64-unknown-linux-gnu
cp target/x86_64-unknown-linux-gnu/release/${NAME} ${RELEASE_DIR}/${NAME}-linux-x86_64
echo "      ✓ releases/${NAME}-linux-x86_64"

echo "[2/6] Linux x86_64 (MUSL)..."
cross build --release --target x86_64-unknown-linux-musl 2>/dev/null || cargo build --release --target x86_64-unknown-linux-musl
cp target/x86_64-unknown-linux-musl/release/${NAME} ${RELEASE_DIR}/${NAME}-linux-x86_64-musl
echo "      ✓ releases/${NAME}-linux-x86_64-musl"

echo "[3/6] Linux ARM64 (GNU)..."
cross build --release --target aarch64-unknown-linux-gnu 2>/dev/null || cargo build --release --target aarch64-unknown-linux-gnu
cp target/aarch64-unknown-linux-gnu/release/${NAME} ${RELEASE_DIR}/${NAME}-linux-arm64
echo "      ✓ releases/${NAME}-linux-arm64"

echo "[4/6] Linux ARM64 (MUSL)..."
cross build --release --target aarch64-unknown-linux-musl 2>/dev/null || cargo build --release --target aarch64-unknown-linux-musl
cp target/aarch64-unknown-linux-musl/release/${NAME} ${RELEASE_DIR}/${NAME}-linux-arm64-musl
echo "      ✓ releases/${NAME}-linux-arm64-musl"

# ==================== Windows ====================
echo "[5/6] Windows x86_64..."
cross build --release --target x86_64-pc-windows-gnu 2>/dev/null || cargo build --release --target x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/${NAME}.exe ${RELEASE_DIR}/${NAME}-windows-x86_64.exe
echo "      ✓ releases/${NAME}-windows-x86_64.exe"

# ==================== macOS ====================
# macOS 需要在 macOS 系统上编译，或者使用 osxcross
# 这里尝试使用 zig 进行交叉编译
if command -v cargo-zigbuild &> /dev/null; then
    echo "[6/6] macOS x86_64 (使用 zig)..."
    cargo zigbuild --release --target x86_64-apple-darwin 2>/dev/null || echo "      ✗ 跳过 (需要 macOS 或 osxcross)"
    if [ -f target/x86_64-apple-darwin/release/${NAME} ]; then
        cp target/x86_64-apple-darwin/release/${NAME} ${RELEASE_DIR}/${NAME}-darwin-x86_64
        echo "      ✓ releases/${NAME}-darwin-x86_64"
    fi

    echo "[7/6] macOS ARM64 (M芯片, 使用 zig)..."
    cargo zigbuild --release --target aarch64-apple-darwin 2>/dev/null || echo "      ✗ 跳过 (需要 macOS 或 osxcross)"
    if [ -f target/aarch64-apple-darwin/release/${NAME} ]; then
        cp target/aarch64-apple-darwin/release/${NAME} ${RELEASE_DIR}/${NAME}-darwin-arm64
        echo "      ✓ releases/${NAME}-darwin-arm64"
    fi
else
    echo "[6/6] macOS (跳过 - 请在 macOS 上手动编译或使用 cargo-zigbuild)"
    # 尝试本地编译（如果在 macOS 上）
    if [[ "$(uname -s)" == "Darwin" ]]; then
        echo "      检测到 macOS，进行本地编译..."
        cargo build --release
        ARCH=$(uname -m)
        if [ "$ARCH" == "arm64" ]; then
            cp target/release/${NAME} ${RELEASE_DIR}/${NAME}-darwin-arm64
            echo "      ✓ releases/${NAME}-darwin-arm64"
        else
            cp target/release/${NAME} ${RELEASE_DIR}/${NAME}-darwin-x86_64
            echo "      ✓ releases/${NAME}-darwin-x86_64"
        fi
    fi
fi

echo ""
echo "=== 编译完成 ==="
echo ""
echo "输出文件:"
ls -lh ${RELEASE_DIR}/
echo ""
echo "平台支持:"
echo "  - Linux x86_64 (GNU/MUSL)"
echo "  - Linux ARM64 (GNU/MUSL)"
echo "  - Windows x86_64"
echo "  - macOS x86_64 (Intel)"
echo "  - macOS ARM64 (M1/M2/M3)"
echo ""
