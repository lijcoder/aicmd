#!/bin/bash

# aicmd 安装脚本
# 支持 Linux 和 macOS

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
CONFIG_DIR="${HOME}/.config/aicmd"

echo "=== aicmd 安装脚本 ==="
echo ""

# 检查 aicmd 文件是否存在
if [[ ! -f "$SCRIPT_DIR/aicmd" ]]; then
    echo "错误: 未找到 aicmd 文件" >&2
    exit 1
fi

# 创建配置目录
echo "创建配置目录: $CONFIG_DIR"
mkdir -p "$CONFIG_DIR"

# 复制配置文件模板（如果不存在）
CONFIG_FILE="$CONFIG_DIR/config"
if [[ ! -f "$CONFIG_FILE" ]]; then
    echo "创建配置文件模板: $CONFIG_FILE"
    cat > "$CONFIG_FILE" << 'EOF'
# aicmd 配置文件
# 请填写您的 API 信息

# OpenAI API 密钥（必填）
# 也可以设置环境变量 AICMD_API_KEY
API_KEY=your_api_key_here

# API 地址（可选，默认使用 OpenAI）
# 也可以设置环境变量 AICMD_API_URL
API_URL=https://api.openai.com/v1/chat/completions

# 模型名称（可选，默认 gpt-3.5-turbo）
# 也可以设置环境变量 AICMD_MODEL
MODEL=gpt-3.5-turbo
EOF
    echo "配置文件已创建，请编辑 $CONFIG_FILE 添加您的 API 密钥"
else
    echo "配置文件已存在: $CONFIG_FILE"
fi

echo ""

# 安装 aicmd
if [[ -w "$INSTALL_DIR" ]]; then
    echo "安装 aicmd 到 $INSTALL_DIR"
    cp "$SCRIPT_DIR/aicmd" "$INSTALL_DIR/aicmd"
    chmod +x "$INSTALL_DIR/aicmd"
else
    echo "需要管理员权限安装到 $INSTALL_DIR"
    sudo cp "$SCRIPT_DIR/aicmd" "$INSTALL_DIR/aicmd"
    sudo chmod +x "$INSTALL_DIR/aicmd"
fi

echo ""
echo "=== 安装完成 ==="
echo ""
echo "使用方法:"
echo "  aicmd '查看当前系统时间'"
echo "  aicmd '查找占用 8080 端口的进程'"
echo "  aicmd -c '如何优化 MySQL 查询性能'"
echo "  cat error.log | aicmd -c '分析这个错误日志'"
echo ""
echo "配置:"
echo "  编辑 $CONFIG_FILE 设置您的 API 密钥"
echo "  或设置环境变量: export AICMD_API_KEY='your_key'"
echo ""

# 检查是否已配置 API 密钥
if grep -q "your_api_key_here" "$CONFIG_FILE" 2>/dev/null; then
    echo "⚠️  警告: 您还未配置 API 密钥"
    echo "   请编辑 $CONFIG_FILE 文件，将 your_api_key_here 替换为您的真实 API 密钥"
fi
