# aicmd

AI 命令行提示工具 - 根据自然语言描述生成 shell 命令，或直接解答问题。

## 功能

1. **命令生成**: 根据描述生成 shell 命令，红色显示，支持交互选项
2. **聊天模式** (`-c`): 直接解答问题，无需确认
3. **管道支持**: 支持从管道读取输入内容
4. **系统检测**: 自动检测操作系统类型和 Shell 类型
5. **纯文本输出**: 命令解释和聊天模式使用纯文本，无 markdown

## 安装

### 方式一：下载预编译二进制文件

从 [Releases](https://github.com/yourname/aicmd/releases) 页面下载对应平台的二进制文件：

| 平台 | 架构 | 文件名 |
|------|------|--------|
| Linux | x86_64 | `aicmd-linux-x86_64` |
| Linux | x86_64 (musl) | `aicmd-linux-x86_64-musl` |
| Linux | ARM64 | `aicmd-linux-arm64` |
| Linux | ARM64 (musl) | `aicmd-linux-arm64-musl` |
| macOS | Intel | `aicmd-darwin-x86_64` |
| macOS | M1/M2/M3 | `aicmd-darwin-arm64` |
| Windows | x86_64 | `aicmd-windows-x86_64.exe` |

```bash
# 下载后赋予执行权限（Linux/macOS）
chmod +x aicmd-*

# 移动到 PATH 目录
mv aicmd-* /usr/local/bin/aicmd
```

### 方式二：从源码编译

```bash
# 克隆项目
git clone <repository>
cd aicmd

# 本地编译
cargo build --release

# 或使用交叉编译脚本
./build-all.sh
```

### 方式三：Cargo 安装

```bash
cargo install --path .
```

## 配置

编辑 `~/.aicmd/config`：

```bash
API_KEY=your_api_key_here
API_URL=https://api.openai.com/v1/chat/completions
MODEL=gpt-3.5-turbo
```

或使用环境变量：

```bash
export AICMD_API_KEY='your_api_key'
export AICMD_API_URL='https://api.openai.com/v1/chat/completions'
export AICMD_MODEL='gpt-3.5-turbo'
```

## 使用示例

### 生成命令

```bash
aicmd "查看当前系统时间"
# 红色显示: date
# 执行命令? [回车/e-执行/d-解释/q-退出]:
```

交互选项：
- `回车` 或 `e` - 执行命令（默认）
- `d` - 解释命令
- `q` - 退出

### 聊天模式

```bash
aicmd -c "如何优化 MySQL 查询性能"
```

### 管道输入

```bash
# 分析日志
cat error.log | aicmd -c "分析这个错误日志"

# 处理文本
echo "hello world" | aicmd "将文本转为大写"
```

## 帮助

```bash
aicmd --help
```

## 系统要求

- **Linux**: kernel 3.2+
- **macOS**: 10.14+ (Intel 或 Apple Silicon)
- **Windows**: Windows 10+

## 交叉编译

本项目支持多平台交叉编译：

```bash
# 安装 cross
cargo install cross

# 编译特定目标
cross build --release --target x86_64-unknown-linux-gnu
cross build --release --target aarch64-unknown-linux-gnu
cross build --release --target x86_64-pc-windows-gnu

# macOS 需要在 macOS 系统上编译，或使用 cargo-zigbuild
cargo install cargo-zigbuild
cargo zigbuild --release --target x86_64-apple-darwin
cargo zigbuild --release --target aarch64-apple-darwin
```

## 自动发布

项目配置了 GitHub Actions，推送标签时自动构建并发布：

```bash
git tag v0.1.0
git push origin v0.1.0
```

## 许可证

MIT
