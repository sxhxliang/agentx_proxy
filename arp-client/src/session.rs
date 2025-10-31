use crate::executor::ExecutorKind;
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, broadcast};
use tokio::time::{Duration, Instant};
use tracing::{info, warn};
use uuid::Uuid;

/// Status of a command session
#[derive(Debug, Clone, PartialEq)]
pub enum SessionStatus {
    Running,
    Completed { exit_code: Option<i32> },
    Failed { error: String },
    Cancelled { reason: String },
}

/// A buffered output line from command execution
#[derive(Debug, Clone)]
pub struct OutputLine {
    pub line_number: usize,
    pub content: String,
    pub timestamp: Instant,
}

/// Session data for a running command
pub struct CommandSession {
    pub session_id: String,
    pub agent_session: Arc<Mutex<Option<(ExecutorKind, String)>>>,
    pub executor_kind: ExecutorKind,
    pub status: Arc<RwLock<SessionStatus>>,
    pub output_buffer: Arc<Mutex<Vec<OutputLine>>>,
    pub last_accessed: Arc<Mutex<Instant>>,
    pub total_lines: Arc<Mutex<usize>>,
    /// Channel for new subscribers to receive output (using broadcast for multiple subscribers)
    pub broadcast_tx: broadcast::Sender<OutputLine>,
    /// Process handle for cancellation (only available while running)
    pub process_handle: Arc<Mutex<Option<tokio::process::Child>>>,
    pub project_path: Arc<RwLock<Option<PathBuf>>>,
}

impl CommandSession {
    pub fn new(session_id: String, executor_kind: ExecutorKind) -> Self {
        // Use broadcast channel with capacity of 1000 messages
        let (tx, _rx) = broadcast::channel(1000);

        CommandSession {
            session_id,
            agent_session: Arc::new(Mutex::new(None)),
            executor_kind,
            status: Arc::new(RwLock::new(SessionStatus::Running)),
            output_buffer: Arc::new(Mutex::new(Vec::new())),
            last_accessed: Arc::new(Mutex::new(Instant::now())),
            total_lines: Arc::new(Mutex::new(0)),
            broadcast_tx: tx,
            process_handle: Arc::new(Mutex::new(None)),
            project_path: Arc::new(RwLock::new(None)),
        }
    }

    /// Add a new output line
    pub async fn add_output(&self, content: String) {
        let mut total = self.total_lines.lock().await;
        *total += 1;
        let line_number = *total;

        let output_line = OutputLine {
            line_number,
            content,
            timestamp: Instant::now(),
        };

        // Add to buffer
        let mut buffer = self.output_buffer.lock().await;
        buffer.push(output_line.clone());

        // Broadcast to any active subscribers
        let _ = self.broadcast_tx.send(output_line);
    }

    /// Mark session as completed
    pub async fn mark_completed(&self, exit_code: Option<i32>) {
        let mut status = self.status.write().await;
        *status = SessionStatus::Completed { exit_code };
        info!("Session {} marked as completed", self.session_id);
    }

    /// Mark session as failed
    pub async fn mark_failed(&self, error: String) {
        let mut status = self.status.write().await;
        *status = SessionStatus::Failed { error };
        warn!("Session {} marked as failed", self.session_id);
    }

    /// Mark session as cancelled
    pub async fn mark_cancelled(&self, reason: String) {
        let mut status = self.status.write().await;
        *status = SessionStatus::Cancelled { reason };
        info!("Session {} marked as cancelled", self.session_id);
    }

    /// Cancel the running process
    pub async fn cancel(&self) -> Result<(), String> {
        let mut process = self.process_handle.lock().await;

        if let Some(ref mut child) = *process {
            match child.kill().await {
                Ok(_) => {
                    info!(
                        "Process for session {} killed successfully",
                        self.session_id
                    );
                    drop(process);
                    self.mark_cancelled("User cancelled".to_string()).await;
                    Ok(())
                }
                Err(e) => {
                    warn!(
                        "Failed to kill process for session {}: {}",
                        self.session_id, e
                    );
                    Err(format!("Failed to kill process: {}", e))
                }
            }
        } else {
            Err("No process handle available (process may have already completed)".to_string())
        }
    }

    /// Set the process handle for this session
    pub async fn set_process_handle(&self, child: tokio::process::Child) {
        let mut handle = self.process_handle.lock().await;
        *handle = Some(child);
    }

    /// Update last accessed time
    pub async fn touch(&self) {
        let mut last_accessed = self.last_accessed.lock().await;
        *last_accessed = Instant::now();
    }

    /// Retrieve current execution status
    pub async fn get_status(&self) -> SessionStatus {
        let status = self.status.read().await;
        status.clone()
    }

    /// Set Claude session ID
    pub async fn set_agent_session(&self, kind: ExecutorKind, agent_session_id: String) {
        let mut agent_session = self.agent_session.lock().await;
        *agent_session = Some((kind, agent_session_id.clone()));
        info!(
            "Session {} linked to {} session: {}",
            self.session_id,
            kind.as_str(),
            agent_session_id
        );
    }

    /// Get agent session info
    pub async fn get_agent_session(&self) -> Option<(ExecutorKind, String)> {
        let agent_session = self.agent_session.lock().await;
        agent_session.clone()
    }

    /// Get all output lines from a specific line number
    pub async fn get_output_from(&self, from_line: usize) -> Vec<OutputLine> {
        let buffer = self.output_buffer.lock().await;
        buffer
            .iter()
            .filter(|line| line.line_number >= from_line)
            .cloned()
            .collect()
    }

    /// Create a new receiver for broadcast updates
    pub fn subscribe(&self) -> broadcast::Receiver<OutputLine> {
        self.broadcast_tx.subscribe()
    }

    pub async fn set_project_path<P>(&self, path: P)
    where
        P: AsRef<Path>,
    {
        let mut project_path = self.project_path.write().await;
        *project_path = Some(path.as_ref().to_path_buf());
    }

    pub async fn get_project_path(&self) -> Option<PathBuf> {
        let project_path = self.project_path.read().await;
        project_path.clone()
    }
}

/// Session manager for tracking command executions
#[derive(Clone)]
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, Arc<CommandSession>>>>,
    agent_session_map: Arc<Mutex<HashMap<(ExecutorKind, String), String>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        let manager = SessionManager {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            agent_session_map: Arc::new(Mutex::new(HashMap::new())),
        };

        // Start cleanup task
        let manager_clone = manager.clone();
        tokio::spawn(async move {
            manager_clone.cleanup_loop().await;
        });

        manager
    }

    /// Create a new session with specific executor
    pub async fn create_session_with_executor(
        &self,
        executor: ExecutorKind,
    ) -> Arc<CommandSession> {
        let session_id = Uuid::new_v4().to_string();
        let session = Arc::new(CommandSession::new(session_id.clone(), executor));

        let mut sessions = self.sessions.lock().await;
        sessions.insert(session_id.clone(), session.clone());

        info!(
            "Created new session: {} with executor: {}",
            session_id,
            executor.as_str()
        );
        session
    }

    /// Create a new session with a custom session ID
    pub async fn create_session_with_id(&self, session_id: String) -> Arc<CommandSession> {
        self.create_session_with_id_and_executor(session_id, ExecutorKind::Claude)
            .await
    }

    /// Create a new session with custom ID and executor
    pub async fn create_session_with_id_and_executor(
        &self,
        session_id: String,
        executor: ExecutorKind,
    ) -> Arc<CommandSession> {
        let session = Arc::new(CommandSession::new(session_id.clone(), executor));

        let mut sessions = self.sessions.lock().await;
        sessions.insert(session_id.clone(), session.clone());

        info!(
            "Created new session with custom ID: {} executor: {}",
            session_id,
            executor.as_str()
        );
        session
    }

    /// Get an existing session
    pub async fn get_session(&self, session_id: &str) -> Option<Arc<CommandSession>> {
        let sessions = self.sessions.lock().await;
        let session = sessions.get(session_id).cloned();

        if let Some(ref s) = session {
            s.touch().await;
        }

        session
    }

    /// Register executor-specific session ID mapping
    pub async fn register_agent_session(
        &self,
        executor_kind: ExecutorKind,
        agent_session_id: String,
        session: &Arc<CommandSession>,
    ) {
        session
            .set_agent_session(executor_kind, agent_session_id.clone())
            .await;

        let mut agent_map = self.agent_session_map.lock().await;
        agent_map.insert(
            (executor_kind, agent_session_id),
            session.session_id.clone(),
        );
    }

    /// Cancel a running session
    pub async fn cancel_session(&self, session_id: &str) -> Result<(), String> {
        let session = self
            .get_session(session_id)
            .await
            .ok_or_else(|| format!("Session not found: {}", session_id))?;

        session.cancel().await
    }

    /// Remove a session
    pub async fn remove_session(&self, session_id: &str) {
        let mut sessions = self.sessions.lock().await;

        // Get the session to retrieve its Claude session ID
        if let Some(session) = sessions.get(session_id) {
            if let Some(agent_info) = session.get_agent_session().await {
                let mut agent_map = self.agent_session_map.lock().await;
                agent_map.remove(&agent_info);
            }
        }

        sessions.remove(session_id);
        info!("Removed session: {}", session_id);
    }

    /// Cleanup old sessions periodically
    async fn cleanup_loop(&self) {
        let cleanup_interval = Duration::from_secs(60); // Check every minute
        let session_timeout = Duration::from_secs(3600); // 1 hour timeout

        loop {
            tokio::time::sleep(cleanup_interval).await;

            let mut sessions = self.sessions.lock().await;
            let mut agent_map = self.agent_session_map.lock().await;
            let now = Instant::now();

            // Find expired sessions
            let expired: Vec<(String, Option<(ExecutorKind, String)>)> = {
                let mut expired = Vec::new();
                for (id, session) in sessions.iter() {
                    let last_accessed = session.last_accessed.lock().await;
                    if now.duration_since(*last_accessed) > session_timeout {
                        let agent_info = session.get_agent_session().await;
                        expired.push((id.clone(), agent_info));
                    }
                }
                expired
            };

            // Remove expired sessions
            for (id, agent_info) in expired {
                sessions.remove(&id);
                if let Some(agent_info) = agent_info {
                    agent_map.remove(&agent_info);
                }
                info!("Cleaned up expired session: {}", id);
            }
        }
    }

    /// Get session statistics
    pub async fn get_stats(&self) -> serde_json::Value {
        let sessions = self.sessions.lock().await;

        let mut running = 0;
        let mut completed = 0;
        let mut failed = 0;
        let mut cancelled = 0;

        for session in sessions.values() {
            let status = session.status.read().await;
            match *status {
                SessionStatus::Running => running += 1,
                SessionStatus::Completed { .. } => completed += 1,
                SessionStatus::Failed { .. } => failed += 1,
                SessionStatus::Cancelled { .. } => cancelled += 1,
            }
        }

        json!({
            "total_sessions": sessions.len(),
            "running": running,
            "completed": completed,
            "failed": failed,
            "cancelled": cancelled
        })
    }

    /// Query session status by session ID
    pub async fn get_session_status(&self, session_id: &str) -> Option<SessionStatus> {
        let session = self.get_session(session_id).await?;
        Some(session.get_status().await)
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
