# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**aicmd** is a command-line AI assistant tool written in Rust that generates shell commands from natural language descriptions. It supports Linux, macOS (Intel/Apple Silicon), and Windows.

## Common Development Commands

### Build
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run directly
cargo run -- "查看当前时间"
cargo run -- -c "如何优化 MySQL"
```

### Cross-Platform Compilation
```bash
# Install cross compilation tool
cargo install cross

# Linux x86_64
cross build --release --target x86_64-unknown-linux-gnu

# Linux ARM64
cross build --release --target aarch64-unknown-linux-gnu

# Windows x86_64
cross build --release --target x86_64-pc-windows-gnu

# macOS (requires macOS host or cargo-zigbuild)
cargo install cargo-zigbuild
cargo zigbuild --release --target x86_64-apple-darwin
cargo zigbuild --release --target aarch64-apple-darwin

# Build all platforms locally
./build-all.sh
```

### Test
```bash
# Run unit tests
cargo test

# Run a specific test
cargo test test_name
```

### Run
```bash
# Command generation mode (interactive)
./target/release/aicmd "查看当前系统时间"

# Chat mode (streaming output)
./target/release/aicmd -c "如何优化 MySQL 查询性能"

# Pipe input
cat error.log | ./target/release/aicmd -c "分析错误日志"
```

## Architecture

### Core Components

**src/main.rs** contains the entire application:

1. **CLI Parsing** (`Args` struct with clap)
   - `-c, --chat-mode`: Enable chat mode (no command generation)
   - `description`: Positional arguments for user query

2. **Configuration** (`Config` struct, `load_config()`)
   - Config file: `~/.aicmd/config` (all platforms)
   - Environment variables override config file: `AICMD_API_KEY`, `AICMD_API_URL`, `AICMD_MODEL`
   - Defaults: API URL points to local proxy `http://127.0.0.1:7888`, model is `deepseek-chat`

3. **Platform Detection**
   - `get_os_name()`: Returns Linux/macOS/Windows based on `std::env::consts::OS`
   - `get_shell_name()`: Platform-specific shell detection
     - Windows: Detects PowerShell vs CMD via environment variables (`PSVersionTable`, `ComSpec`)
     - Unix: Parses `$SHELL` environment variable

4. **API Communication**
   - `call_api()` (non-streaming): Used for command generation. Returns complete response at once.
   - `call_api_stream()` (streaming): Used for chat mode and command explanation. Implements Server-Sent Events (SSE) parsing for typewriter effect with 10ms delay per character.
   - Both use OpenAI-compatible chat completions API with temperature fixed at 0.3

5. **Prompt Generation**
   - `generate_command_prompt()`: System prompt includes OS and shell type for context-aware command generation
   - `explain_command_prompt()`: For explaining generated commands
   - `chat_mode_prompt()`: For general Q&A without command generation
   - All prompts explicitly forbid markdown formatting for terminal readability

6. **Command Execution**
   - Windows: Uses `cmd /C` to execute commands
   - Unix: Uses `sh -c` to execute commands
   - Interactive loop with options: `[回车/e-执行/d-解释/q-退出]`

### Data Flow

1. Parse CLI args with clap
2. Read stdin if piped input detected (via `atty`)
3. Load config (file → environment variables → defaults)
4. Construct appropriate prompt based on mode (command/chat/explain)
5. Call LLM API asynchronously (streaming or non-streaming based on mode)
6. Display result:
   - Command mode: Commands displayed in red via `colored`
   - Chat/Explain mode: Typewriter-style streaming output
7. Interactive execution loop (command mode only)

### Output Modes

| Mode | API Call | Output Style |
|------|----------|--------------|
| Command Generation | `call_api()` (non-streaming) | Complete command, red color |
| Chat Mode (`-c`) | `call_api_stream()` | Typewriter effect, plain text |
| Command Explanation (`d`) | `call_api_stream()` | Typewriter effect, plain text |

### Key Dependencies

- `clap`: CLI argument parsing with derive macros
- `reqwest` + `tokio`: Async HTTP client and runtime
- `serde` + `serde_json`: JSON serialization for API requests/responses
- `colored`: Terminal color output
- `dirs`: Cross-platform config directory detection
- `atty`: TTY detection for pipe input
- `anyhow`: Error handling

## Supported Platforms

| Platform | Architecture | Target Triple |
|----------|--------------|---------------|
| Linux | x86_64 | `x86_64-unknown-linux-gnu/musl` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu/musl` |
| macOS | Intel | `x86_64-apple-darwin` |
| macOS | Apple Silicon | `aarch64-apple-darwin` |
| Windows | x86_64 | `x86_64-pc-windows-msvc/gnu` |

## Configuration

Config file format (`~/.config/aicmd/config`):
```bash
API_KEY=your_api_key_here
API_URL=https://api.openai.com/v1/chat/completions
MODEL=gpt-3.5-turbo
```

## Release Process

GitHub Actions workflow (`.github/workflows/release.yml`) automatically builds and releases on tag push:

```bash
git tag v0.1.0
git push origin v0.1.0
```

This triggers builds for all supported platforms and creates a GitHub release with artifacts.

## Notes

- Shell version (`aicmd` bash script) is preserved but no longer maintained; Rust version is primary
- The tool is designed for Chinese users - all system prompts are in Chinese
- Generated commands are displayed in red; explanations and chat responses are plain text (no markdown)
- Default API endpoint assumes a local proxy; users should configure their own API key and endpoint
- Streaming output uses SSE (Server-Sent Events) format with 10ms delay per character for typewriter effect
