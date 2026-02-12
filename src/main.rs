use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufRead, Read, Write};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

const DEFAULT_API_URL: &str = "http://127.0.0.1:7888/proxy/direct/aigc/chat/completions";
const DEFAULT_API_KEY: &str = "sk-aicmd";
const DEFAULT_MODEL: &str = "deepseek-chat";

#[derive(Parser)]
#[command(name = "aicmd")]
#[command(about = "AI 命令行提示工具")]
struct Args {
    /// 聊天/解答模式，直接输出答案，不生成命令
    #[arg(short = 'c', long)]
    chat_mode: bool,

    /// 描述或问题
    description: Vec<String>,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Deserialize)]
struct ApiError {
    error: ErrorDetail,
}

#[derive(Deserialize)]
struct ErrorDetail {
    message: String,
}

// 流式响应结构
#[derive(Deserialize)]
struct StreamResponse {
    choices: Vec<StreamChoice>,
}

#[derive(Deserialize)]
struct StreamChoice {
    delta: Delta,
}

#[derive(Deserialize)]
struct Delta {
    #[serde(default)]
    content: Option<String>,
}

struct Config {
    api_key: String,
    api_url: String,
    model: String,
}

fn get_os_name() -> String {
    match std::env::consts::OS {
        "linux" => "Linux".to_string(),
        "macos" => "macOS".to_string(),
        "windows" => "Windows".to_string(),
        os => os.to_string(),
    }
}

fn get_shell_name() -> String {
    #[cfg(target_os = "windows")]
    {
        // Windows 检测 PowerShell 或 CMD
        if let Ok(ps_version) = std::env::var("PSVersionTable") {
            return "powershell".to_string();
        }
        if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
            if term_program.to_lowercase().contains("powershell") {
                return "powershell".to_string();
            }
        }
        // 检查可执行文件路径
        if let Ok(comspec) = std::env::var("ComSpec") {
            if comspec.to_lowercase().contains("cmd") {
                return "cmd".to_string();
            }
        }
        return "powershell".to_string(); // Windows 默认使用 PowerShell
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("SHELL")
            .ok()
            .and_then(|s| s.split('/').last().map(|s| s.to_string()))
            .unwrap_or_else(|| "bash".to_string())
    }
}

fn load_config() -> Config {
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let config_file = home_dir.join(".aicmd").join("config");

    let mut api_key = DEFAULT_API_KEY.to_string();
    let mut api_url = DEFAULT_API_URL.to_string();
    let mut model = DEFAULT_MODEL.to_string();

    // 尝试读取配置文件
    if let Ok(content) = std::fs::read_to_string(&config_file) {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "API_KEY" => api_key = value.to_string(),
                    "API_URL" => api_url = value.to_string(),
                    "MODEL" => model = value.to_string(),
                    _ => {}
                }
            }
        }
    }

    // 环境变量优先级更高
    if let Ok(env_key) = std::env::var("AICMD_API_KEY") {
        api_key = env_key;
    }
    if let Ok(env_url) = std::env::var("AICMD_API_URL") {
        api_url = env_url;
    }
    if let Ok(env_model) = std::env::var("AICMD_MODEL") {
        model = env_model;
    }

    Config {
        api_key,
        api_url,
        model,
    }
}

// 非流式 API 调用（用于命令生成）
async fn call_api(client: &Client, config: &Config, system_prompt: &str, user_prompt: &str) -> Result<String> {
    let request = ChatRequest {
        model: config.model.clone(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            },
        ],
        temperature: 0.3,
        stream: Some(false),
    };

    let response = client
        .post(&config.api_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", config.api_key))
        .json(&request)
        .send()
        .await
        .context("API 请求失败")?;

    let status = response.status();
    let text = response.text().await.context("读取响应失败")?;

    if !status.is_success() {
        // 尝试解析错误
        if let Ok(error) = serde_json::from_str::<ApiError>(&text) {
            anyhow::bail!("API 错误: {}", error.error.message);
        }
        anyhow::bail!("API 请求失败: {}", text);
    }

    // 检查是否有 error 字段
    if text.contains("\"error\"") {
        if let Ok(error) = serde_json::from_str::<ApiError>(&text) {
            anyhow::bail!("API 错误: {}", error.error.message);
        }
    }

    let chat_response: ChatResponse = serde_json::from_str(&text)
        .context("解析响应失败")?;

    Ok(chat_response
        .choices
        .get(0)
        .map(|c| c.message.content.clone())
        .unwrap_or_default())
}

// 流式 API 调用（用于聊天和解释，打字机效果）
async fn call_api_stream(
    client: &Client,
    config: &Config,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String> {
    let request = ChatRequest {
        model: config.model.clone(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            },
        ],
        temperature: 0.3,
        stream: Some(true),
    };

    let mut response = client
        .post(&config.api_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", config.api_key))
        .json(&request)
        .send()
        .await
        .context("API 请求失败")?;

    if !response.status().is_success() {
        let text = response.text().await.context("读取响应失败")?;
        if let Ok(error) = serde_json::from_str::<ApiError>(&text) {
            anyhow::bail!("API 错误: {}", error.error.message);
        }
        anyhow::bail!("API 请求失败: {}", text);
    }

    let mut full_content = String::new();
    let mut stdout = io::stdout();

    while let Some(chunk) = response.chunk().await.context("读取流失败")? {
        let text = String::from_utf8_lossy(&chunk);

        // 处理 SSE 格式的数据
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line == "data: [DONE]" {
                continue;
            }

            if let Some(data) = line.strip_prefix("data: ") {
                // 尝试解析 JSON
                if let Ok(stream_resp) = serde_json::from_str::<StreamResponse>(data) {
                    if let Some(choice) = stream_resp.choices.get(0) {
                        if let Some(content) = &choice.delta.content {
                            print!("{}", content);
                            stdout.flush()?;
                            full_content.push_str(content);
                            // 小延迟产生打字机效果
                            sleep(Duration::from_millis(10)).await;
                        }
                    }
                }
            }
        }
    }

    println!(); // 最后换行
    Ok(full_content)
}

fn generate_command_prompt(description: &str, stdin_content: Option<&str>) -> (String, String) {
    let os_name = get_os_name();
    let shell_name = get_shell_name();

    let system_prompt = format!(
        "你是一个命令行专家。用户当前使用的是 {} 操作系统，{} shell。根据用户的描述生成最适合当前环境的 shell 命令。只输出命令本身，不要有任何解释、注释或 markdown 格式。命令应该简洁、安全且可执行。",
        os_name, shell_name
    );

    let user_prompt = if let Some(content) = stdin_content {
        format!(
            "描述: {}\n\n输入内容:\n{}\n\n请生成处理上述内容的命令:",
            description, content
        )
    } else {
        format!("描述: {}", description)
    };

    (system_prompt, user_prompt)
}

fn explain_command_prompt(command: &str) -> (String, String) {
    let os_name = get_os_name();
    let shell_name = get_shell_name();

    let system_prompt = format!(
        "你是一个命令行专家。用户当前使用的是 {} 操作系统，{} shell。解释给定的 shell 命令，包括每个参数的含义、命令的作用以及使用注意事项。用中文回答。禁止使用任何 markdown 格式（如代码块、加粗、列表等），使用纯文本格式输出，确保内容直观易读。",
        os_name, shell_name
    );

    let user_prompt = format!("请解释以下命令:\n\n{}", command);

    (system_prompt, user_prompt)
}

fn chat_mode_prompt(question: &str, stdin_content: Option<&str>) -> (String, String) {
    let os_name = get_os_name();
    let shell_name = get_shell_name();

    let system_prompt = format!(
        "你是一个 helpful 的助手。用户当前使用的是 {} 操作系统，{} shell。请简洁、准确地回答用户的问题。用中文回答。禁止使用任何 markdown 格式（如代码块、加粗、列表、标题等），使用纯文本格式输出，确保内容直观易读。",
        os_name, shell_name
    );

    let user_prompt = if let Some(content) = stdin_content {
        format!("问题: {}\n\n输入内容:\n{}", question, content)
    } else {
        question.to_string()
    };

    (system_prompt, user_prompt)
}

// 从终端设备读取用户输入（即使有管道输入）
fn read_from_terminal() -> Result<String> {
    #[cfg(target_os = "windows")]
    {
        // Windows: 使用 CONIN$ 读取控制台输入
        match File::open("CONIN$") {
            Ok(file) => {
                let mut reader = io::BufReader::new(file);
                let mut buf = String::new();
                reader.read_line(&mut buf)?;
                Ok(buf.trim().to_string())
            }
            Err(_) => {
                // 如果无法打开 CONIN$，回退到 stdin
                let mut buf = String::new();
                io::stdin().read_line(&mut buf)?;
                Ok(buf.trim().to_string())
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Unix/Linux/macOS: 使用 /dev/tty 读取终端输入
        match File::open("/dev/tty") {
            Ok(file) => {
                let mut reader = io::BufReader::new(file);
                let mut buf = String::new();
                reader.read_line(&mut buf)?;
                Ok(buf.trim().to_string())
            }
            Err(_) => {
                // 如果无法打开 /dev/tty（比如在某些 CI 环境），回退到 stdin
                let mut buf = String::new();
                io::stdin().read_line(&mut buf)?;
                Ok(buf.trim().to_string())
            }
        }
    }
}

fn show_help() {
    println!("Usage: aicmd [选项] [描述]");
    println!("       cat file | aicmd [选项] [描述]");
    println!();
    println!("AI 命令行提示工具 - 根据描述生成命令或解答问题");
    println!();
    println!("选项:");
    println!("    -c          聊天/解答模式，直接输出答案，不生成命令");
    println!("    -h, --help  显示帮助信息");
    println!();
    println!("示例:");
    println!("    aicmd \"查看当前系统时间\"");
    println!("    aicmd \"查找占用 8080 端口的进程\"");
    println!("    aicmd -c \"如何优化 MySQL 查询性能\"");
    println!("    cat error.log | aicmd -c \"分析这个错误日志\"");
    println!();
    println!("配置:");
    println!("    在 ~/.aicmd/config 中配置 API 信息:");
    println!("    API_KEY=your_api_key");
    println!("    API_URL=https://api.openai.com/v1/chat/completions");
    println!("    MODEL=gpt-3.5-turbo");
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();


    // 读取管道输入
    let stdin_content = if atty::isnt(atty::Stream::Stdin) {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).ok();
        if buffer.trim().is_empty() {
            None
        } else {
            Some(buffer)
        }
    } else {
        None
    };

    // 合并描述参数
    let description = args.description.join(" ");

    if description.is_empty() && stdin_content.is_none() {
        eprintln!("错误: 请提供描述或使用管道输入");
        eprintln!("使用 -h 或 --help 查看帮助");
        std::process::exit(1);
    }

    let config = load_config();
    let client = Client::new();

    // 聊天模式 - 使用流式输出
    if args.chat_mode {
        let (system_prompt, user_prompt) = chat_mode_prompt(&description, stdin_content.as_deref());
        let _response = call_api_stream(&client, &config, &system_prompt, &user_prompt).await?;
        return Ok(());
    }

    // 命令生成模式
    let (system_prompt, user_prompt) = generate_command_prompt(&description, stdin_content.as_deref());
    let generated_command = call_api(&client, &config, &system_prompt, &user_prompt).await?;

    if generated_command.is_empty() {
        anyhow::bail!("无法生成命令");
    }

    // 红色显示命令
    println!("  {}", generated_command.red());

    // 检测是否有管道输入 - 如果有，需要从终端设备读取用户输入
    let has_pipe_input = stdin_content.is_some();

    // 循环处理用户选择
    loop {
        print!("执行命令? [回车/e-执行/d-解释/q-退出]: ");
        io::stdout().flush()?;

        let choice = if has_pipe_input {
            // 有管道输入时，从 /dev/tty (Unix) 或 CONIN$ (Windows) 读取
            read_from_terminal()?
        } else {
            // 没有管道输入时，直接从 stdin 读取
            let mut buf = String::new();
            io::stdin().read_line(&mut buf)?;
            buf.trim().to_string()
        };

        // 默认空输入为执行
        let choice = if choice.is_empty() { "e".to_string() } else { choice };

        match choice.as_str() {
            "e" | "E" | "exec" | "EXEC" => {
                println!();
                // 执行命令 - 根据平台选择 shell
                #[cfg(target_os = "windows")]
                let shell = ("cmd", vec!["/C"]);
                #[cfg(not(target_os = "windows"))]
                let shell = ("sh", vec!["-c"]);

                let status = std::process::Command::new(shell.0)
                    .args(&shell.1)
                    .arg(&generated_command)
                    .status()?;
                if !status.success() {
                    eprintln!("命令执行失败");
                }
                break;
            }
            "d" | "D" => {
                let (system_prompt, user_prompt) = explain_command_prompt(&generated_command);
                // 使用流式输出解释命令
                let _explanation = call_api_stream(&client, &config, &system_prompt, &user_prompt).await?;
                println!();
                // 解释后重新显示命令
                println!("  {}", generated_command.red());
            }
            "q" | "Q" | "quit" | "QUIT" => {
                std::process::exit(0);
            }
            _ => {
                println!("无效选项，请重新选择");
                std::process::exit(0);
            }
        }
    }

    Ok(())
}
