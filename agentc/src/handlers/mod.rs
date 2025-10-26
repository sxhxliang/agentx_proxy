pub mod proxy;
pub mod session;

use crate::config::ClientConfig;
use crate::session::SessionManager;
use std::sync::Arc;

/// Shared state for handlers
#[derive(Clone)]
pub struct HandlerState {
    pub config: Arc<ClientConfig>,
    pub session_manager: SessionManager,
}

impl HandlerState {
    pub fn new(config: ClientConfig) -> Self {
        let session_manager = SessionManager::new();

        HandlerState {
            config: Arc::new(config),
            session_manager,
        }
    }
}
