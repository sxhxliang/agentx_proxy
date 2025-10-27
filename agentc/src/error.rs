use thiserror::Error;

/// Unified error type for agentc
#[derive(Debug, Error)]
pub enum AgentcError {
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("Connection error: {0}")]
    ConnectionError(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Executor error: {0}")]
    ExecutorError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Port validation error: {0}")]
    PortValidationError(String),

    #[error("Path validation error: {0}")]
    PathValidationError(String),

    #[error("Command execution error: {0}")]
    CommandExecutionError(String),

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, AgentcError>;

// Implement conversion from anyhow::Error for compatibility
impl From<anyhow::Error> for AgentcError {
    fn from(err: anyhow::Error) -> Self {
        AgentcError::Unknown(err.to_string())
    }
}

// Implement conversion from String for convenience
impl From<String> for AgentcError {
    fn from(err: String) -> Self {
        AgentcError::Unknown(err)
    }
}

impl From<&str> for AgentcError {
    fn from(err: &str) -> Self {
        AgentcError::Unknown(err.to_string())
    }
}
