use crate::agentx::{claude, codex, gemini};
use crate::executor::{
    ClaudeOptions, CodexOptions, ExecutorKind, ExecutorOptions, GeminiOptions, build_command,
    parse_bool_str,
};
use crate::handlers::HandlerState;
use crate::router::HandlerContext;
use crate::session::{CommandSession, SessionStatus};
use anyhow::{Result, anyhow};
use common::http::{HttpResponse, json_error};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::oneshot;
use tracing::{error, info, warn};

/// Unified handler for session operations
pub async fn handle_session(ctx: HandlerContext, state: HandlerState) -> Result<HttpResponse> {
    let proxy_conn_id = ctx.proxy_conn_id.clone();
    let method = &ctx.request.method;

    // Check if this is a session-specific request (has session_id)
    let session_id = ctx.path_params.get("session_id").cloned();

    match (method, session_id) {
        // POST /api/sessions - Create new session
        (common::http::HttpMethod::POST, None) => handle_create_session(ctx, state).await,
        // GET /api/sessions/{session_id} - Get session or reconnect
        (common::http::HttpMethod::GET, Some(session_id)) => {
            handle_get_session(ctx, state, &session_id).await
        }
        // DELETE /api/sessions/{session_id} - Cancel or delete session
        (common::http::HttpMethod::DELETE, Some(session_id)) => {
            handle_delete_session(ctx, state, &session_id).await
        }
        // GET /api/sessions - List all sessions (future implementation)
        (common::http::HttpMethod::GET, None) => {
            info!("('{}') List sessions request", proxy_conn_id);
            let mut stream = ctx.stream;
            let _ = json_error(501, "List sessions not yet implemented")
                .send(&mut stream)
                .await;
            Ok(HttpResponse::ok())
        }
        _ => {
            let mut stream = ctx.stream;
            let _ = json_error(405, "Method not allowed")
                .send(&mut stream)
                .await;
            Ok(HttpResponse::ok())
        }
    }
}

/// Handle session creation (POST /api/sessions)
async fn handle_create_session(ctx: HandlerContext, state: HandlerState) -> Result<HttpResponse> {
    let proxy_conn_id = ctx.proxy_conn_id.clone();
    let request = &ctx.request;

    // Parse parameters from body or query
    let body_json = if request.method == common::http::HttpMethod::POST {
        request.body_as_json().unwrap_or(json!({}))
    } else {
        json!({})
    };

    let prompt = body_json["prompt"]
        .as_str()
        .or_else(|| request.query_param("prompt").map(|s| s.as_str()))
        .unwrap_or("")
        .to_string();

    let project_path = body_json["project_path"]
        .as_str()
        .or_else(|| request.query_param("project_path").map(|s| s.as_str()))
        .unwrap_or("")
        .to_string();

    // Validate required parameters
    if prompt.is_empty() || project_path.is_empty() {
        let mut stream = ctx.stream;
        let _ = json_error(
            400,
            "prompt and project_path are required and cannot be empty",
        )
        .send(&mut stream)
        .await;
        return Ok(HttpResponse::ok());
    }

    // Parse executor options
    let (executor_options, error) = parse_executor_options(&body_json, request);

    if let Some(error_message) = error {
        let mut stream = ctx.stream;
        let _ = json_error(400, error_message).send(&mut stream).await;
        return Ok(HttpResponse::ok());
    }

    let executor_options = executor_options.unwrap();

    info!(
        "('{}') Creating session with executor: {}",
        proxy_conn_id,
        executor_options.kind().as_str()
    );

    // Create channel to receive session after it's created
    let (session_tx, session_rx) = oneshot::channel();

    // Start command execution in background
    let session_manager_clone = state.session_manager.clone();
    tokio::spawn(async move {
        if let Err(e) = execute_command(
            session_tx,
            prompt,
            project_path,
            executor_options,
            session_manager_clone,
        )
        .await
        {
            error!("Command execution failed: {}", e);
        }
    });

    // Wait for session to be created
    let session = match session_rx.await {
        Ok(Some(s)) => s,
        Ok(None) => {
            error!(
                "('{}') Failed to create session: no output received",
                proxy_conn_id
            );
            let mut stream = ctx.stream;
            let _ = json_error(500, "Failed to create session: command produced no output")
                .send(&mut stream)
                .await;
            return Ok(HttpResponse::ok());
        }
        Err(_) => {
            error!(
                "('{}') Failed to receive session from execute_command",
                proxy_conn_id
            );
            let mut stream = ctx.stream;
            let _ = json_error(500, "Internal error: failed to create session")
                .send(&mut stream)
                .await;
            return Ok(HttpResponse::ok());
        }
    };

    let session_id = session.session_id.clone();
    info!("('{}') Session created: {}", proxy_conn_id, session_id);

    // Stream output to client
    stream_session_output(ctx, session, 0).await
}

/// Handle session retrieval or reconnection (GET /api/sessions/{session_id})
async fn handle_get_session(
    ctx: HandlerContext,
    state: HandlerState,
    session_id: &str,
) -> Result<HttpResponse> {
    let proxy_conn_id = &ctx.proxy_conn_id;
    let from_line = ctx
        .request
        .query_param("from_line")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);

    let in_memory_session = state.session_manager.get_session(session_id).await;

    // Determine which executor to use for loading history
    let executor_kind = if let Some(session) = &in_memory_session {
        session.executor_kind
    } else {
        ctx.request
            .query_param("executor")
            .and_then(|value| ExecutorKind::from_str(value))
            .unwrap_or(ExecutorKind::Claude)
    };

    let historical_messages = load_history_for_executor(executor_kind, session_id).await;

    if in_memory_session.is_none() && historical_messages.is_none() {
        warn!("('{}') Session not found: {}", proxy_conn_id, session_id);
        let mut stream = ctx.stream;
        let _ = json_error(404, "Session not found").send(&mut stream).await;
        return Ok(HttpResponse::ok());
    }

    stream_unified_session(ctx, in_memory_session, historical_messages, from_line).await
}

/// Handle session cancellation without deletion (POST /api/sessions/{session_id}/cancel)
pub async fn handle_cancel_session(
    ctx: HandlerContext,
    state: HandlerState,
) -> Result<HttpResponse> {
    let session_id = match ctx.path_params.get("session_id") {
        Some(v) if !v.is_empty() => v.clone(),
        _ => {
            let mut stream = ctx.stream;
            let _ = json_error(400, "session_id is required")
                .send(&mut stream)
                .await;
            return Ok(HttpResponse::ok());
        }
    };

    let mut stream = ctx.stream;

    if let Some(_session) = state.session_manager.get_session(&session_id).await {
        match state.session_manager.cancel_session(&session_id).await {
            Ok(_) => {
                let body = json!({"type": "session_cancelled", "session_id": session_id});
                let _ = HttpResponse::ok().json(&body).send(&mut stream).await;
            }
            Err(e) => {
                let _ = json_error(500, format!("Failed to cancel session: {}", e))
                    .send(&mut stream)
                    .await;
            }
        }
    } else {
        let _ = json_error(404, "Session not found or not running")
            .send(&mut stream)
            .await;
    }

    Ok(HttpResponse::ok())
}

/// Handle session deletion/cancellation (DELETE /api/sessions/{session_id})
async fn handle_delete_session(
    ctx: HandlerContext,
    state: HandlerState,
    session_id: &str,
) -> Result<HttpResponse> {
    let proxy_conn_id = &ctx.proxy_conn_id;
    let mut stream = ctx.stream;

    // Check if session is in memory (active)
    if let Some(session) = state.session_manager.get_session(session_id).await {
        let status = session.get_status().await;

        match status {
            SessionStatus::Running => {
                // Cancel the running session
                info!(
                    "('{}') Cancelling running session: {}",
                    proxy_conn_id, session_id
                );

                match state.session_manager.cancel_session(session_id).await {
                    Ok(_) => {
                        let body = json!({
                            "type": "session_cancelled",
                            "session_id": session_id
                        });
                        let _ = HttpResponse::ok().json(&body).send(&mut stream).await;
                        Ok(HttpResponse::ok())
                    }
                    Err(e) => {
                        let _ = json_error(500, format!("Failed to cancel session: {}", e))
                            .send(&mut stream)
                            .await;
                        Ok(HttpResponse::ok())
                    }
                }
            }
            _ => {
                // Session is completed/failed, remove from memory
                state.session_manager.remove_session(session_id).await;
                let body = json!({
                    "type": "session_removed",
                    "session_id": session_id
                });
                let _ = HttpResponse::ok().json(&body).send(&mut stream).await;
                Ok(HttpResponse::ok())
            }
        }
    } else {
        // Not in memory, try to delete from file system
        info!(
            "('{}') Deleting historical session: {}",
            proxy_conn_id, session_id
        );

        let requested_executor = ctx
            .request
            .query_param("executor")
            .and_then(|value| ExecutorKind::from_str(value));

        match delete_history_for_executor(requested_executor, session_id).await {
            Ok(_) => {
                let body = json!({
                    "type": "session_deleted",
                    "session_id": session_id
                });
                let _ = HttpResponse::ok().json(&body).send(&mut stream).await;
                Ok(HttpResponse::ok())
            }
            Err(e) => {
                let status = if e.contains("not found") { 404 } else { 500 };
                let _ = json_error(status, e).send(&mut stream).await;
                Ok(HttpResponse::ok())
            }
        }
    }
}

async fn load_history_for_executor(executor: ExecutorKind, session_id: &str) -> Option<Vec<Value>> {
    match executor {
        ExecutorKind::Claude => claude::load_session_by_id(session_id.to_string())
            .await
            .ok(),
        ExecutorKind::Codex => codex::load_session_by_id(session_id.to_string()).await.ok(),
        ExecutorKind::Gemini => gemini::load_session_by_id(session_id.to_string())
            .await
            .ok(),
    }
}

async fn delete_history_for_executor(
    executor_override: Option<ExecutorKind>,
    session_id: &str,
) -> Result<(), String> {
    if let Some(executor) = executor_override {
        return delete_history_by_kind(executor, session_id).await;
    }

    for executor in [
        ExecutorKind::Claude,
        ExecutorKind::Codex,
        ExecutorKind::Gemini,
    ] {
        match delete_history_by_kind(executor, session_id).await {
            Ok(_) => return Ok(()),
            Err(err) if err.contains("not found") => continue,
            Err(err) => return Err(err),
        }
    }

    Err(format!(
        "Session {} not found in supported executors",
        session_id
    ))
}

async fn delete_history_by_kind(executor: ExecutorKind, session_id: &str) -> Result<(), String> {
    match executor {
        ExecutorKind::Claude => claude::delete_session_by_id(session_id.to_string()).await,
        ExecutorKind::Codex => codex::delete_session_by_id(session_id.to_string()).await,
        ExecutorKind::Gemini => gemini::delete_session_by_id(session_id.to_string()).await,
    }
}

// Helper to get string parameter from body or query
fn get_param(body: &Value, request: &common::http::HttpRequest, key: &str) -> Option<String> {
    body[key]
        .as_str()
        .or_else(|| request.query_param(key).map(|s| s.as_str()))
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
}

// Helper to get array parameter from body
fn get_array_param(body: &Value, key: &str) -> Option<Vec<String>> {
    body.get(key)?.as_array().map(|arr| {
        arr.iter()
            .filter_map(|item| item.as_str().map(String::from))
            .collect()
    })
}

// Helper to validate enum values
fn validate_enum(value: &str, valid: &[&str], name: &str) -> Result<(), String> {
    if valid.contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "Invalid {}: {}. Valid options: {}",
            name,
            value,
            valid.join(", ")
        ))
    }
}

/// Parse executor options from request
fn parse_executor_options(
    body_json: &Value,
    request: &common::http::HttpRequest,
) -> (Option<ExecutorOptions>, Option<String>) {
    let executor_kind = get_param(body_json, request, "executor")
        .and_then(|s| ExecutorKind::from_str(&s))
        .unwrap_or(ExecutorKind::Claude);

    let options = match executor_kind {
        ExecutorKind::Claude => {
            let resume = get_param(body_json, request, "resume");
            let model = get_param(body_json, request, "model");
            let permission_mode = get_param(body_json, request, "permission_mode");
            let allowed_tools = get_array_param(body_json, "allowed_tools");

            if let Some(ref mode) = permission_mode {
                if let Err(e) = validate_enum(
                    mode,
                    &["acceptEdits", "bypassPermissions", "default", "plan"],
                    "permission_mode",
                ) {
                    return (None, Some(e));
                }
            }

            ExecutorOptions::Claude(ClaudeOptions {
                resume,
                model,
                permission_mode,
                allowed_tools,
            })
        }
        ExecutorKind::Codex => {
            let model = get_param(body_json, request, "model");
            let resume_last = match body_json.get("resume_last") {
                Some(Value::Bool(b)) => *b,
                Some(Value::String(s)) => parse_bool_str(s).unwrap_or(false),
                Some(Value::Number(n)) => n.as_i64() == Some(1),
                None => request
                    .query_param("resume_last")
                    .and_then(|s| parse_bool_str(s.trim()))
                    .unwrap_or(false),
                _ => {
                    return (
                        None,
                        Some("resume_last must be a boolean, string, or number".to_string()),
                    );
                }
            };

            ExecutorOptions::Codex(CodexOptions { model, resume_last })
        }
        ExecutorKind::Gemini => {
            let approval_mode = get_param(body_json, request, "approval_mode");

            if let Some(ref mode) = approval_mode {
                if let Err(e) =
                    validate_enum(mode, &["default", "auto_edit", "yolo"], "approval_mode")
                {
                    return (None, Some(e));
                }
            }

            ExecutorOptions::Gemini(GeminiOptions { approval_mode })
        }
    };

    (Some(options), None)
}

/// Execute the command and store output in session
async fn execute_command(
    session_tx: oneshot::Sender<Option<Arc<CommandSession>>>,
    prompt: String,
    project_path: String,
    executor_options: ExecutorOptions,
    session_manager: crate::session::SessionManager,
) -> Result<()> {
    // Build command
    let mut cmd = match build_command(&executor_options, &prompt, &project_path) {
        Ok(cmd) => cmd,
        Err(e) => {
            error!("Failed to build command: {}", e);
            let _ = session_tx.send(None);
            return Err(e);
        }
    };

    // Spawn the process
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let error = format!("Failed to spawn command: {}", e);
            error!("{}", error);
            let _ = session_tx.send(None);
            return Err(anyhow!("Failed to spawn command"));
        }
    };

    // Get stdout for streaming
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("Failed to get stdout"))?;
    let mut stdout_reader = BufReader::new(stdout);

    info!("Command started, reading output...");

    // Read first line to extract session ID and create session
    let mut first_line = String::new();
    let bytes_read = match stdout_reader.read_line(&mut first_line).await {
        Ok(n) => n,
        Err(e) => {
            error!("Error reading first line: {}", e);
            let _ = session_tx.send(None);
            return Err(anyhow!("Failed to read first line"));
        }
    };

    if bytes_read == 0 {
        error!("Command produced no output");
        let _ = session_tx.send(None);
        return Err(anyhow!("Command produced no output"));
    }

    // Trim the first line
    let trimmed_first_line = first_line.trim_end_matches(['\r', '\n']);

    // Try to parse as JSON and extract session_id field
    let session = match serde_json::from_str::<Value>(trimmed_first_line) {
        Ok(json_value) => {
            if let Some(session_id) = json_value.get("session_id").and_then(|v| v.as_str()) {
                info!("Extracted session ID: {}", session_id);
                let session = session_manager
                    .create_session_with_id_and_executor(
                        session_id.to_string(),
                        executor_options.kind(),
                    )
                    .await;
                session_manager
                    .register_agent_session(
                        executor_options.kind(),
                        session_id.to_string(),
                        &session,
                    )
                    .await;
                session.set_project_path(PathBuf::from(&project_path)).await;
                session
            } else {
                error!("First line JSON missing 'session_id' field");
                let _ = session_tx.send(None);
                return Err(anyhow!("First line JSON missing 'session_id' field"));
            }
        }
        Err(e) => {
            error!("Failed to parse first line as JSON: {}", e);
            let _ = session_tx.send(None);
            return Err(anyhow!("Failed to parse first line as JSON"));
        }
    };

    let session_id = &session.session_id;
    info!("[Session {}] Created session", session_id);

    // Store process handle for cancellation
    session.set_process_handle(child).await;

    // Add first line to session buffer
    session.add_output(trimmed_first_line.to_string()).await;

    // Send session back to handle_create_session
    if session_tx.send(Some(session.clone())).is_err() {
        error!("[Session {}] Failed to send session to handler", session_id);
        return Err(anyhow!("Failed to send session to handler"));
    }

    // Continue reading remaining output lines
    loop {
        let mut line = String::new();
        let bytes_read = match stdout_reader.read_line(&mut line).await {
            Ok(n) => n,
            Err(e) => {
                error!("[Session {}] Error reading stdout: {}", session_id, e);
                break;
            }
        };

        if bytes_read == 0 {
            break; // EOF
        }

        // Trim the line
        let trimmed_line = line.trim_end_matches(['\r', '\n']);

        // Add to session buffer
        session.add_output(trimmed_line.to_string()).await;
    }

    // Retrieve process handle and wait for completion
    let mut process_handle = session.process_handle.lock().await;
    if let Some(child) = process_handle.as_mut() {
        let exit_status = child.wait().await?;
        let exit_code = exit_status.code();

        info!(
            "[Session {}] Command completed with exit code: {:?}",
            session_id, exit_code
        );

        // Mark session as completed
        drop(process_handle);
        session.mark_completed(exit_code).await;
    }

    Ok(())
}

/// Unified SSE streaming for all session types
async fn stream_unified_session(
    ctx: HandlerContext,
    session: Option<Arc<CommandSession>>,
    historical_messages: Option<Vec<serde_json::Value>>,
    from_line: usize,
) -> Result<HttpResponse> {
    // let proxy_conn_id = &ctx.proxy_conn_id;
    let mut stream = ctx.stream;

    // Send SSE headers
    stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, PUT, DELETE, PATCH, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type, Authorization\r\n\r\n").await?;
    stream.flush().await?;

    // Send session info
    let session_id = session
        .as_ref()
        .map(|s| s.session_id.as_str())
        .unwrap_or("unknown");

    info!("[Session {}] Sending session info", session_id);
    // Stream historical messages first
    if let Some(messages) = historical_messages {
        for (idx, msg) in messages.iter().enumerate() {
            let line = idx + 1;
            if line < from_line {
                continue;
            }

            if stream
                .write_all(format!("data: {}\n\n", msg).as_bytes())
                .await
                .is_err()
            {
                return Ok(HttpResponse::ok());
            }
            stream.flush().await?;
        }
    }

    // Stream live session if exists
    let Some(session) = session else {
        let completion = json!({"type":"completion","success":true});
        let _ = stream
            .write_all(format!("data: {}\n\n", completion).as_bytes())
            .await;
        return Ok(HttpResponse::ok());
    };

    let mut current_line = *session.total_lines.lock().await;
    drop(session.total_lines.lock().await);

    // Send buffered output
    for line in session.get_output_from(from_line).await {
        // let event = json!({"type":"output","line":line.line_number,"content":line.content});
        if stream
            .write_all(format!("data: {}\n\n", line.content).as_bytes())
            .await
            .is_err()
        {
            return Ok(HttpResponse::ok());
        }
        stream.flush().await?;
    }

    // Poll for new output
    loop {
        let status = session.status.read().await.clone();
        let is_complete = !matches!(status, SessionStatus::Running);

        for line in session.get_output_from(current_line + 1).await {
            current_line = line.line_number;

            if stream
                .write_all(format!("data: {}\n\n", line.content).as_bytes())
                .await
                .is_err()
            {
                return Ok(HttpResponse::ok());
            }
            stream.flush().await?;
        }

        if is_complete {
            let completion = match status {
                SessionStatus::Completed { exit_code } => {
                    json!({"type":"completion","success":true,"exit_code":exit_code,"total_lines":current_line})
                }
                SessionStatus::Failed { error } => {
                    json!({"type":"completion","success":false,"error":error,"total_lines":current_line})
                }
                SessionStatus::Cancelled { reason } => {
                    json!({"type":"completion","success":false,"cancelled":true,"reason":reason,"total_lines":current_line})
                }
                _ => unreachable!(),
            };
            let _ = stream
                .write_all(format!("data: {}\n\n", completion).as_bytes())
                .await;
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    Ok(HttpResponse::ok())
}

/// Stream session output to client via SSE (used by create_session)
async fn stream_session_output(
    ctx: HandlerContext,
    session: Arc<CommandSession>,
    from_line: usize,
) -> Result<HttpResponse> {
    stream_unified_session(ctx, Some(session), None, from_line).await
}
