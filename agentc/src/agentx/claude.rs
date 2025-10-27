use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a project in the ~/.claude/projects directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// The project ID (derived from the directory name)
    pub id: String,
    /// The original project path (decoded from the directory name)
    pub path: String,
    /// List of session IDs (JSONL file names without extension)
    pub sessions: Vec<String>,
    /// Unix timestamp when the project directory was created
    pub created_at: u64,
    /// Unix timestamp of the most recent session (if any)
    pub most_recent_session: Option<u64>,
}

/// Represents a session with its metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// The session ID (UUID)
    pub id: String,
    /// The project ID this session belongs to
    pub project_id: String,
    /// The project path
    pub project_path: String,
    /// Optional todo data associated with this session
    pub todo_data: Option<serde_json::Value>,
    /// Unix timestamp when the session file was created
    pub created_at: u64,
    /// First user message content (if available)
    pub first_message: Option<String>,
    /// Timestamp of the first user message (if available)
    pub message_timestamp: Option<String>,
    /// Total number of messages in the session
    pub message_count: usize,
    /// Session status ('completed', 'ongoing', 'pending', or 'failed')
    pub status: String,
    /// Total duration in seconds (from first to last message timestamp)
    pub total_duration: Option<f64>,
}

/// Represents a message entry in the JSONL file
#[derive(Debug, Deserialize)]
struct JsonlEntry {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    entry_type: Option<String>,
    message: Option<MessageContent>,
    timestamp: Option<String>,
}

/// Represents the message content
#[derive(Debug, Deserialize)]
struct MessageContent {
    role: Option<String>,
    content: Option<String>,
}

/// Represents a working directory entry for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingDirectory {
    /// The full filesystem path
    pub path: String,
    /// Short name (last two path components)
    #[serde(rename = "shortname")]
    pub short_name: String,
    /// Last modification date in ISO 8601 format
    #[serde(rename = "lastDate")]
    pub last_date: String,
    /// Number of conversation sessions
    #[serde(rename = "conversationCount")]
    pub conversation_count: usize,
}

/// Gets the path to the ~/.claude directory
fn get_claude_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .context("Could not find home directory")?
        .join(".claude")
        .canonicalize()
        .context("Could not find ~/.claude directory")
}

/// Gets the actual project path by reading the cwd from the JSONL entries
pub fn get_project_path_from_sessions(project_dir: &PathBuf) -> Result<String, String> {
    // Try to read any JSONL file in the directory
    let entries = fs::read_dir(project_dir)
        .map_err(|e| format!("Failed to read project directory: {}", e))?;

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                // Read the JSONL file and find the first line with a valid cwd
                if let Ok(file) = fs::File::open(&path) {
                    let reader = BufReader::new(file);
                    // Check first few lines instead of just the first line
                    // Some session files may have null cwd in the first line
                    for line in reader.lines().take(10) {
                        if let Ok(line_content) = line {
                            // Parse the JSON and extract cwd
                            if let Ok(json) =
                                serde_json::from_str::<serde_json::Value>(&line_content)
                            {
                                if let Some(cwd) = json.get("cwd").and_then(|v| v.as_str()) {
                                    if !cwd.is_empty() {
                                        return Ok(cwd.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Err("Could not determine project path from session files".to_string())
}

/// Decodes a project directory name back to its original path
/// The directory names in ~/.claude/projects are encoded paths
/// DEPRECATED: Use get_project_path_from_sessions instead when possible
fn decode_project_path(encoded: &str) -> String {
    // This is a fallback - the encoding isn't reversible when paths contain hyphens
    // For example: -Users-mufeedvh-dev-jsonl-viewer could be /Users/mufeedvh/dev/jsonl-viewer
    // or /Users/mufeedvh/dev/jsonl/viewer
    encoded.replace('-', "/")
}

/// Extracts session metadata from a JSONL file
/// Returns (first_message, first_timestamp, message_count, total_duration, status)
async fn extract_session_metadata(
    jsonl_path: &PathBuf,
) -> (Option<String>, Option<String>, usize, Option<f64>, String) {
    let _session_id = jsonl_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());

    let status_from_runtime: Option<String> = None;

    let file = match fs::File::open(jsonl_path) {
        Ok(file) => file,
        Err(_) => {
            let status = status_from_runtime.unwrap_or_else(|| "pending".to_string());
            return (None, None, 0, None, status);
        }
    };

    let reader = BufReader::new(file);
    let mut first_message: Option<String> = None;
    let mut first_timestamp: Option<String> = None;
    let mut first_timestamp_parsed: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut last_timestamp_parsed: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut message_count = 0;

    for line in reader.lines() {
        if let Ok(line) = line {
            if let Ok(entry) = serde_json::from_str::<JsonlEntry>(&line) {
                // Count all messages
                if entry.message.is_some() {
                    message_count += 1;
                }

                // Extract first valid user message
                if first_message.is_none() {
                    if let Some(message) = &entry.message {
                        if message.role.as_deref() == Some("user") {
                            if let Some(content) = &message.content {
                                // Skip caveat messages
                                if content.contains("Caveat: The messages below were generated by the user while running local commands") {
                                    continue;
                                }

                                // Skip command tags
                                if content.starts_with("<command-name>")
                                    || content.starts_with("<local-command-stdout>")
                                {
                                    continue;
                                }

                                // Found a valid user message
                                first_message = Some(content.clone());
                                first_timestamp = entry.timestamp.clone();
                            }
                        }
                    }
                }

                // Track timestamps for duration calculation
                if let Some(timestamp_str) = &entry.timestamp {
                    if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(timestamp_str) {
                        let utc_time = parsed.with_timezone(&chrono::Utc);
                        if first_timestamp_parsed.is_none() {
                            first_timestamp_parsed = Some(utc_time);
                        }
                        last_timestamp_parsed = Some(utc_time);
                    }
                }
            }
        }
    }

    // Calculate total duration
    let total_duration =
        if let (Some(first), Some(last)) = (first_timestamp_parsed, last_timestamp_parsed) {
            let duration = last.signed_duration_since(first);
            Some(duration.num_milliseconds() as f64 / 1000.0) // Convert to seconds
        } else {
            None
        };

    // Determine status using runtime session manager if available, fallback to metadata
    let status = if let Some(status) = status_from_runtime {
        status
    } else if message_count == 0 {
        "pending".to_string()
    } else if let Ok(metadata) = fs::metadata(jsonl_path) {
        if let Ok(modified) = metadata.modified() {
            let elapsed = SystemTime::now()
                .duration_since(modified)
                .unwrap_or_default();
            // If modified within last 3 s, consider it ongoing
            if elapsed.as_secs() < 3 {
                "ongoing".to_string()
            } else {
                "completed".to_string()
            }
        } else {
            "completed".to_string()
        }
    } else {
        "completed".to_string()
    };

    (
        first_message,
        first_timestamp,
        message_count,
        total_duration,
        status,
    )
}
/// Lists all projects in the ~/.claude/projects directory
pub async fn list_projects() -> Result<Vec<Project>, String> {
    let claude_dir = get_claude_dir().map_err(|e| e.to_string())?;
    let projects_dir = claude_dir.join("projects");
    tracing::info!("Listing projects from {:?}", claude_dir);
    if !projects_dir.exists() {
        tracing::warn!("Projects directory does not exist: {:?}", projects_dir);
        return Ok(Vec::new());
    }

    let mut projects = Vec::new();

    // Read all directories in the projects folder
    let entries = fs::read_dir(&projects_dir)
        .map_err(|e| format!("Failed to read projects directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| "Invalid directory name".to_string())?;

            // Get directory creation time
            let metadata = fs::metadata(&path)
                .map_err(|e| format!("Failed to read directory metadata: {}", e))?;

            let created_at = metadata
                .created()
                .or_else(|_| metadata.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH)
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            // Get the actual project path from JSONL files
            let project_path = match get_project_path_from_sessions(&path) {
                Ok(path) => path,
                Err(e) => {
                    tracing::warn!("Failed to get project path from sessions for {}: {}, falling back to decode", dir_name, e);
                    decode_project_path(dir_name)
                }
            };

            // List all JSONL files (sessions) in this project directory
            let mut sessions = Vec::new();
            let mut most_recent_session: Option<u64> = None;

            if let Ok(session_entries) = fs::read_dir(&path) {
                for session_entry in session_entries.flatten() {
                    let session_path = session_entry.path();
                    if session_path.is_file()
                        && session_path.extension().and_then(|s| s.to_str()) == Some("jsonl")
                    {
                        if let Some(session_id) = session_path.file_stem().and_then(|s| s.to_str())
                        {
                            sessions.push(session_id.to_string());

                            // Track the most recent session timestamp
                            if let Ok(metadata) = fs::metadata(&session_path) {
                                let modified = metadata
                                    .modified()
                                    .unwrap_or(SystemTime::UNIX_EPOCH)
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();

                                most_recent_session = Some(match most_recent_session {
                                    Some(current) => current.max(modified),
                                    None => modified,
                                });
                            }
                        }
                    }
                }
            }

            projects.push(Project {
                id: dir_name.to_string(),
                path: project_path,
                sessions,
                created_at,
                most_recent_session,
            });
        }
    }

    // Sort projects by most recent session activity, then by creation time
    projects.sort_by(|a, b| {
        // First compare by most recent session
        match (a.most_recent_session, b.most_recent_session) {
            (Some(a_time), Some(b_time)) => b_time.cmp(&a_time),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => b.created_at.cmp(&a.created_at),
        }
    });

    tracing::info!("Found {} projects", projects.len());
    Ok(projects)
}

/// Gets sessions for a specific project
pub async fn get_project_sessions(project_id: String) -> Result<Vec<Session>, String> {
    tracing::info!("Getting sessions for project: {}", project_id);

    let claude_dir = get_claude_dir().map_err(|e| e.to_string())?;
    let project_dir = claude_dir.join("projects").join(&project_id);
    let todos_dir = claude_dir.join("todos");

    if !project_dir.exists() {
        return Err(format!("Project directory not found: {}", project_id));
    }

    // Get the actual project path from JSONL files
    let project_path = match get_project_path_from_sessions(&project_dir) {
        Ok(path) => path,
        Err(e) => {
            tracing::warn!(
                "Failed to get project path from sessions for {}: {}, falling back to decode",
                project_id,
                e
            );
            decode_project_path(&project_id)
        }
    };

    let mut sessions = Vec::new();

    // Read all JSONL files in the project directory
    let entries = fs::read_dir(&project_dir)
        .map_err(|e| format!("Failed to read project directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            if let Some(session_id) = path.file_stem().and_then(|s| s.to_str()) {
                // Get file creation time
                let metadata = fs::metadata(&path)
                    .map_err(|e| format!("Failed to read file metadata: {}", e))?;

                let created_at = metadata
                    .created()
                    .or_else(|_| metadata.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH)
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                // Extract session metadata including message count, duration, and status
                let (first_message, message_timestamp, message_count, total_duration, status) =
                    extract_session_metadata(&path).await;

                // Try to load associated todo data
                let todo_path = todos_dir.join(format!("{}.json", session_id));
                let todo_data = if todo_path.exists() {
                    fs::read_to_string(&todo_path)
                        .ok()
                        .and_then(|content| serde_json::from_str(&content).ok())
                } else {
                    None
                };

                sessions.push(Session {
                    id: session_id.to_string(),
                    project_id: project_id.clone(),
                    project_path: project_path.clone(),
                    todo_data,
                    created_at,
                    first_message,
                    message_timestamp,
                    message_count,
                    status,
                    total_duration,
                });
            }
        }
    }

    // Sort sessions by creation time (newest first)
    sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    tracing::info!(
        "Found {} sessions for project {}",
        sessions.len(),
        project_id
    );
    Ok(sessions)
}

/// Loads the JSONL history for a specific session by session ID only
/// This function searches across all projects to find the session file
pub async fn load_session_by_id(session_id: String) -> Result<Vec<serde_json::Value>, String> {
    tracing::info!("Loading session history for session ID: {}", session_id);

    let claude_dir = get_claude_dir().map_err(|e| e.to_string())?;
    let projects_dir = claude_dir.join("projects");

    if !projects_dir.exists() {
        return Err("Projects directory does not exist".to_string());
    }

    // Remove .jsonl extension if provided
    let clean_session_id = session_id.trim_end_matches(".jsonl");

    // Search through all project directories for the session file
    let entries = fs::read_dir(&projects_dir)
        .map_err(|e| format!("Failed to read projects directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            // Check if the session file exists in this project directory
            let session_path = path.join(format!("{}.jsonl", clean_session_id));

            if session_path.exists() {
                tracing::info!("Found session file at: {:?}", session_path);

                let file = fs::File::open(&session_path)
                    .map_err(|e| format!("Failed to open session file: {}", e))?;

                let reader = BufReader::new(file);
                let mut messages = Vec::new();

                for line in reader.lines().flatten() {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                        messages.push(json);
                    }
                }

                return Ok(messages);
            }
        }
    }

    Err(format!(
        "Session file not found for session ID: {}",
        clean_session_id
    ))
}

/// Removes a session JSONL file (and its todo, if present) by session ID
pub async fn delete_session_by_id(session_id: String) -> Result<(), String> {
    tracing::info!("Deleting session with ID: {}", session_id);

    let claude_dir = get_claude_dir().map_err(|e| e.to_string())?;
    let projects_dir = claude_dir.join("projects");
    let todos_dir = claude_dir.join("todos");

    if !projects_dir.exists() {
        return Err("Projects directory does not exist".to_string());
    }

    let clean_session_id = session_id.trim_end_matches(".jsonl");
    let session_filename = format!("{}.jsonl", clean_session_id);

    let entries = fs::read_dir(&projects_dir)
        .map_err(|e| format!("Failed to read projects directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let session_path = path.join(&session_filename);

        if session_path.exists() {
            fs::remove_file(&session_path)
                .map_err(|e| format!("Failed to delete session file: {}", e))?;
            tracing::info!("Removed session file at {:?}", session_path);

            let todo_path = todos_dir.join(format!("{}.json", clean_session_id));
            if todo_path.exists() {
                match fs::remove_file(&todo_path) {
                    Ok(_) => tracing::info!("Removed session todo file at {:?}", todo_path),
                    Err(e) => {
                        tracing::warn!("Failed to delete session todo file {:?}: {}", todo_path, e)
                    }
                }
            }

            return Ok(());
        }
    }

    Err(format!(
        "Session file not found for session ID: {}",
        clean_session_id
    ))
}

/// Gets all sessions across all projects, sorted by time (newest first)
///
/// # Arguments
/// * `limit` - Maximum number of sessions to return (optional)
/// * `offset` - Number of sessions to skip (optional)
/// * `project_path` - Filter sessions by project path (optional)
pub async fn get_all_sessions(
    limit: Option<usize>,
    offset: Option<usize>,
    project_path: Option<String>,
) -> Result<Vec<Session>, String> {
    tracing::info!(
        "Getting all sessions across all projects (limit: {:?}, offset: {:?}, project_path: {:?})",
        limit,
        offset,
        project_path
    );

    let claude_dir = get_claude_dir().map_err(|e| e.to_string())?;
    let projects_dir = claude_dir.join("projects");
    let todos_dir = claude_dir.join("todos");

    if !projects_dir.exists() {
        tracing::warn!("Projects directory does not exist: {:?}", projects_dir);
        return Ok(Vec::new());
    }

    let mut all_sessions = Vec::new();

    // Read all project directories
    let project_entries = fs::read_dir(&projects_dir)
        .map_err(|e| format!("Failed to read projects directory: {}", e))?;

    for project_entry in project_entries {
        let project_entry =
            project_entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let project_path = project_entry.path();

        if !project_path.is_dir() {
            continue;
        }

        let project_id = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| "Invalid directory name".to_string())?
            .to_string();

        // Get the actual project path from JSONL files
        let project_real_path = match get_project_path_from_sessions(&project_path) {
            Ok(path) => path,
            Err(e) => {
                tracing::warn!(
                    "Failed to get project path from sessions for {}: {}, falling back to decode",
                    project_id,
                    e
                );
                decode_project_path(&project_id)
            }
        };

        // Read all session files in this project
        let session_entries = match fs::read_dir(&project_path) {
            Ok(entries) => entries,
            Err(e) => {
                tracing::warn!("Failed to read project directory {}: {}", project_id, e);
                continue;
            }
        };

        for session_entry in session_entries {
            let session_entry = match session_entry {
                Ok(entry) => entry,
                Err(e) => {
                    tracing::warn!("Failed to read session entry: {}", e);
                    continue;
                }
            };

            let session_path = session_entry.path();

            if session_path.is_file()
                && session_path.extension().and_then(|s| s.to_str()) == Some("jsonl")
            {
                if let Some(session_id) = session_path.file_stem().and_then(|s| s.to_str()) {
                    // Get file metadata
                    let metadata = match fs::metadata(&session_path) {
                        Ok(meta) => meta,
                        Err(e) => {
                            tracing::warn!("Failed to read metadata for {}: {}", session_id, e);
                            continue;
                        }
                    };

                    let created_at = metadata
                        .created()
                        .or_else(|_| metadata.modified())
                        .unwrap_or(SystemTime::UNIX_EPOCH)
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();

                    // Extract session metadata including message count, duration, and status
                    let (first_message, message_timestamp, message_count, total_duration, status) =
                        extract_session_metadata(&session_path).await;

                    // Try to load associated todo data
                    let todo_path = todos_dir.join(format!("{}.json", session_id));
                    let todo_data = if todo_path.exists() {
                        fs::read_to_string(&todo_path)
                            .ok()
                            .and_then(|content| serde_json::from_str(&content).ok())
                    } else {
                        None
                    };

                    all_sessions.push(Session {
                        id: session_id.to_string(),
                        project_id: project_id.clone(),
                        project_path: project_real_path.clone(),
                        todo_data,
                        created_at,
                        first_message,
                        message_timestamp,
                        message_count,
                        status,
                        total_duration,
                    });
                }
            }
        }
    }

    // Sort sessions by creation time (newest first)
    all_sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    // Filter by project path if specified
    if let Some(ref filter_path) = project_path {
        all_sessions.retain(|session| session.project_path == *filter_path);
    }

    let total_count = all_sessions.len();
    tracing::info!(
        "Found {} sessions across all projects (before pagination)",
        total_count
    );

    // Apply pagination
    let offset_val = offset.unwrap_or(0);
    let limit_val = limit.unwrap_or(usize::MAX);

    let paginated_sessions: Vec<Session> = all_sessions
        .into_iter()
        .skip(offset_val)
        .take(limit_val)
        .collect();

    tracing::info!(
        "Returning {} sessions (offset: {}, limit: {:?})",
        paginated_sessions.len(),
        offset_val,
        limit
    );

    Ok(paginated_sessions)
}

/// Gets all project working directories with metadata
pub async fn get_working_directories() -> Result<Vec<WorkingDirectory>, String> {
    tracing::info!("Getting all project working directories");

    let claude_dir = get_claude_dir().map_err(|e| e.to_string())?;
    let projects_dir = claude_dir.join("projects");

    if !projects_dir.exists() {
        tracing::warn!("Projects directory does not exist: {:?}", projects_dir);
        return Ok(Vec::new());
    }

    let mut directories = Vec::new();

    // Read all project directories
    let project_entries = fs::read_dir(&projects_dir)
        .map_err(|e| format!("Failed to read projects directory: {}", e))?;

    for project_entry in project_entries {
        let project_entry =
            project_entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let project_path = project_entry.path();

        if !project_path.is_dir() {
            continue;
        }

        let project_id = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| "Invalid directory name".to_string())?
            .to_string();

        // Get the actual project path from JSONL files
        let project_real_path = match get_project_path_from_sessions(&project_path) {
            Ok(path) => path,
            Err(e) => {
                tracing::warn!(
                    "Failed to get project path from sessions for {}: {}, skipping",
                    project_id,
                    e
                );
                continue;
            }
        };

        // Calculate short name (last two path components)
        let path_components: Vec<&str> = project_real_path.split('/').collect();
        let short_name = if path_components.len() >= 2 {
            format!(
                "{}/{}",
                path_components[path_components.len() - 2],
                path_components[path_components.len() - 1]
            )
        } else {
            project_real_path.clone()
        };

        // Count sessions and get most recent timestamp
        let mut session_count = 0;
        let mut most_recent_timestamp: Option<u64> = None;

        if let Ok(session_entries) = fs::read_dir(&project_path) {
            for session_entry in session_entries.flatten() {
                let session_path = session_entry.path();
                if session_path.is_file()
                    && session_path.extension().and_then(|s| s.to_str()) == Some("jsonl")
                {
                    session_count += 1;

                    // Track most recent modification
                    if let Ok(metadata) = fs::metadata(&session_path) {
                        let modified = metadata
                            .modified()
                            .unwrap_or(SystemTime::UNIX_EPOCH)
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();

                        most_recent_timestamp = Some(match most_recent_timestamp {
                            Some(current) => current.max(modified),
                            None => modified,
                        });
                    }
                }
            }
        }

        // Convert timestamp to ISO 8601 format
        let last_date = if let Some(timestamp) = most_recent_timestamp {
            let datetime = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp);
            // Format as ISO 8601: YYYY-MM-DDTHH:MM:SSZ
            let datetime_chrono = chrono::DateTime::<chrono::Utc>::from(datetime);
            datetime_chrono.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        } else {
            // If no sessions, use directory creation time
            let metadata = fs::metadata(&project_path)
                .map_err(|e| format!("Failed to read directory metadata: {}", e))?;
            let created = metadata
                .created()
                .or_else(|_| metadata.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            let datetime_chrono = chrono::DateTime::<chrono::Utc>::from(created);
            datetime_chrono.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        };

        directories.push(WorkingDirectory {
            path: project_real_path,
            short_name,
            last_date,
            conversation_count: session_count,
        });
    }

    // Sort by last_date (newest first)
    directories.sort_by(|a, b| b.last_date.cmp(&a.last_date));

    tracing::info!("Found {} working directories", directories.len());
    Ok(directories)
}
