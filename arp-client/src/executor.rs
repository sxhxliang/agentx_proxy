use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command as TokioCommand;
use tracing::info;

/// Executor type for command execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutorKind {
    Claude,
    Codex,
    #[serde(rename = "gemini")]
    Gemini, // Future support
}

impl ExecutorKind {
    pub fn from_str(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "claude" => Some(Self::Claude),
            "codex" => Some(Self::Codex),
            "gemini" => Some(Self::Gemini),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutorKind::Claude => "claude",
            ExecutorKind::Codex => "codex",
            ExecutorKind::Gemini => "gemini",
        }
    }

    /// Get the storage directory for this executor's session files
    pub fn storage_dir(&self) -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;

        let dir_name = match self {
            ExecutorKind::Claude => ".claude",
            ExecutorKind::Codex => ".codex",
            ExecutorKind::Gemini => ".gemini",
        };

        Ok(home.join(dir_name))
    }
}

/// Options for Claude executor
#[derive(Debug, Clone, Default)]
pub struct ClaudeOptions {
    pub resume: Option<String>, // session_id to resume
    pub model: Option<String>,
    pub permission_mode: Option<String>, // "acceptEdits" | "bypassPermissions" | "default" | "plan"
    pub allowed_tools: Option<Vec<String>>,
}

/// Options for Codex executor
#[derive(Debug, Clone, Default)]
pub struct CodexOptions {
    pub model: Option<String>,
    pub resume_last: bool,
}

/// Options for Gemini executor
#[derive(Debug, Clone, Default)]
pub struct GeminiOptions {
    pub approval_mode: Option<String>, // "default" | "auto_edit" | "yolo"
}

/// Options for command execution
#[derive(Debug, Clone)]
pub enum ExecutorOptions {
    Claude(ClaudeOptions),
    Codex(CodexOptions),
    Gemini(GeminiOptions),
}

impl ExecutorOptions {
    pub fn kind(&self) -> ExecutorKind {
        match self {
            ExecutorOptions::Claude(_) => ExecutorKind::Claude,
            ExecutorOptions::Codex(_) => ExecutorKind::Codex,
            ExecutorOptions::Gemini(_) => ExecutorKind::Gemini,
        }
    }
}

/// Build a command for the specified executor
pub fn build_command(
    executor_options: &ExecutorOptions,
    prompt: &str,
    project_path: &str,
) -> Result<TokioCommand> {
    match executor_options {
        ExecutorOptions::Claude(options) => build_claude_command(prompt, project_path, options),
        ExecutorOptions::Codex(options) => build_codex_command(prompt, project_path, options),
        ExecutorOptions::Gemini(options) => build_gemini_command(prompt, project_path, options),
    }
}

/// Build Claude command
fn build_claude_command(
    prompt: &str,
    project_path: &str,
    options: &ClaudeOptions,
) -> Result<TokioCommand> {
    let claude_path = find_claude_binary()?;

    let mut cmd = TokioCommand::new(claude_path);

    // Basic arguments
    cmd.arg("-p");
    cmd.arg(prompt);
    cmd.arg("--output-format");
    cmd.arg("stream-json");
    cmd.arg("--verbose");

    // Resume session if specified
    if let Some(ref session_id) = options.resume {
        cmd.arg("--resume");
        cmd.arg(session_id);
        info!("Claude resuming session: {}", session_id);
    }

    // Model selection
    if let Some(ref model) = options.model {
        cmd.arg("--model");
        cmd.arg(model);
        info!("Claude using model: {}", model);
    }

    // Permission mode
    if let Some(ref perm_mode) = options.permission_mode {
        cmd.arg("--permission-mode");
        cmd.arg(perm_mode);
        info!("Claude permission mode: {}", perm_mode);
    } else {
        // Default to bypass permissions for automated execution
        cmd.arg("--dangerously-skip-permissions");
    }

    // Allowed tools
    if let Some(ref tools) = options.allowed_tools {
        for tool in tools {
            cmd.arg("--allowedTools");
            cmd.arg(tool);
        }
        info!("Claude allowed tools: {:?}", tools);
    }

    cmd.current_dir(project_path);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    Ok(cmd)
}

/// Build Codex command
fn build_codex_command(
    prompt: &str,
    project_path: &str,
    options: &CodexOptions,
) -> Result<TokioCommand> {
    let codex_binary = "codex";

    if which::which(codex_binary).is_err() {
        return Err(anyhow!("Codex binary not found in system PATH"));
    }

    info!("Using Codex binary: {}", codex_binary);

    if let Some(model) = options.model.as_deref() {
        info!("Codex model: {}", model);
    }
    if options.resume_last {
        info!("Resuming last Codex session");
    }

    let mut cmd = TokioCommand::new(codex_binary);
    cmd.arg("exec");
    cmd.arg("--json");
    cmd.arg("--sandbox");
    cmd.arg("danger-full-access");
    cmd.arg("--full-auto");

    if let Some(model) = options.model.as_deref() {
        cmd.arg("--model");
        cmd.arg(model);
    }

    if options.resume_last {
        cmd.arg("resume");
        cmd.arg("--last");
    }

    cmd.arg(prompt);
    cmd.current_dir(project_path);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    Ok(cmd)
}

/// Build Gemini command
fn build_gemini_command(
    prompt: &str,
    project_path: &str,
    options: &GeminiOptions,
) -> Result<TokioCommand> {
    let gemini_binary = "gemini";

    if which::which(gemini_binary).is_err() {
        return Err(anyhow!("Gemini binary not found in system PATH"));
    }

    info!("Using Gemini binary: {}", gemini_binary);

    let mut cmd = TokioCommand::new(gemini_binary);
    cmd.arg("exec");
    cmd.arg("--json");

    // Approval mode
    if let Some(ref approval_mode) = options.approval_mode {
        match approval_mode.as_str() {
            "default" => {
                // No additional flag, default behavior
            }
            "auto_edit" => {
                cmd.arg("--approval-mode");
                cmd.arg("auto_edit");
            }
            "yolo" => {
                cmd.arg("--approval-mode");
                cmd.arg("yolo");
            }
            _ => {
                return Err(anyhow!(
                    "Invalid approval_mode: {}. Valid options: default, auto_edit, yolo",
                    approval_mode
                ));
            }
        }
        info!("Gemini approval mode: {}", approval_mode);
    }

    cmd.arg(prompt);
    cmd.current_dir(project_path);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    Ok(cmd)
}

/// Find Claude binary on the system
#[cfg(windows)]
fn find_claude_binary() -> Result<String> {
    // First try the bundled binary (same location as Tauri app uses)
    let bundled_binary = "src-tauri/binaries/claude-code-x86_64-pc-windows-msvc.exe";
    if std::path::Path::new(bundled_binary).exists() {
        info!(
            "[find_claude_binary] Using bundled binary: {}",
            bundled_binary
        );
        return Ok(bundled_binary.to_string());
    }

    // Fall back to system installation paths
    let mut candidates: Vec<String> = vec![
        "claude.exe".to_string(),
        "claude.cmd".to_string(),
        "claude-code.exe".to_string(),
    ];

    // Add user-specific paths
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        candidates.extend(vec![
            format!("{}\\.local\\bin\\claude.exe", user_profile),
            format!("{}\\.local\\bin\\claude.cmd", user_profile),
            format!("{}\\AppData\\Roaming\\npm\\claude.cmd", user_profile),
            format!("{}\\.yarn\\bin\\claude.cmd", user_profile),
            format!("{}\\.bun\\bin\\claude.exe", user_profile),
        ]);
    }

    // Add ProgramFiles paths
    if let Ok(program_files) = std::env::var("ProgramFiles") {
        candidates.push(format!("{}\\Claude Code\\claude.exe", program_files));
    }

    if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
        candidates.push(format!("{}\\Claude Code\\claude.exe", program_files_x86));
    }

    for candidate in &candidates {
        if which::which(candidate).is_ok() {
            info!("[find_claude_binary] Using system binary: {}", candidate);
            return Ok(candidate.to_string());
        }
    }

    Err(anyhow!(
        "Claude binary not found in bundled location or system paths"
    ))
}

#[cfg(not(windows))]
fn find_claude_binary() -> Result<String> {
    let candidates = vec!["claude", "claude-code"];

    for candidate in &candidates {
        if which::which(candidate).is_ok() {
            info!("[find_claude_binary] Using system binary: {}", candidate);
            return Ok(candidate.to_string());
        }
    }

    Err(anyhow!("Claude binary not found in system PATH"))
}

/// Parse a boolean string value
pub fn parse_bool_str(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}
