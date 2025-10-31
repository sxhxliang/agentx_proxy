use http;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use std::time::Duration;

// ==================== Constants ====================

/// Default polling interval for permission status checks
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Maximum timeout for permission requests (1 hour)
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60 * 60);

/// Default ARP server port
const DEFAULT_ARP_PORT: &str = "17004";

/// Default streaming ID when none is provided
const DEFAULT_STREAMING_ID: &str = "unknown";

// ==================== Permission Management Structures ====================

/// Arguments for the approval_prompt tool
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ApprovalPromptArgs {
    /// The tool requesting permission
    pub tool_name: String,
    /// The input for the tool
    pub input: serde_json::Value,
}

/// Request body for permission notification
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PermissionNotificationRequest {
    tool_name: String,
    tool_input: serde_json::Value,
    streaming_id: String,
}

/// Response from permission notification
#[derive(Debug, serde::Deserialize)]
struct PermissionNotificationResponse {
    success: bool,
    id: String,
}

/// Permission status
#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
enum PermissionStatus {
    Pending,
    Approved,
    Denied,
}

/// Permission object
#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Permission {
    id: String,
    status: PermissionStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    modified_input: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deny_reason: Option<String>,
}

/// Response from permissions list endpoint
#[derive(Debug, serde::Deserialize)]
struct PermissionsResponse {
    permissions: Vec<Permission>,
}

/// Permission approval response
#[derive(Debug, serde::Serialize)]
struct ApprovalResponse {
    behavior: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "updatedInput")]
    updated_input: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

/// Permission Manager for handling approval prompts
///
/// This struct manages permission requests from MCP clients to a ARP (Conversational User Interface) server.
/// It handles the complete lifecycle of permission requests including:
/// - Sending permission notifications to the ARP server
/// - Polling for approval/denial decisions with configurable timeout
/// - Processing and responding with appropriate approval responses
///
/// # Configuration
///
/// The manager can be configured via constructor parameters or environment variables:
/// - `ARP_SERVER_URL`: Base URL of the ARP server (default: http://localhost:17004)
/// - `ARP_SERVER_PORT`: Port of the ARP server (used if full URL not provided)
/// - `ARP_STREAMING_ID`: Unique identifier for the streaming session (default: "unknown")
#[derive(Clone)]
pub struct PermissionManager {
    /// Base URL of the ARP server for API communication
    arp_server_url: String,
    /// Unique identifier for the current streaming session
    arp_streaming_id: String,
    /// HTTP client with optimized timeout settings for API communication
    http_client: reqwest::Client,
    /// Tool router for handling MCP tool registration
    tool_router: ToolRouter<PermissionManager>,
}

#[tool_router]
impl PermissionManager {
    /// Create a new PermissionManager with custom configuration
    ///
    /// # Arguments
    ///
    /// * `arp_server_url` - Optional base URL for the ARP server. If not provided,
    ///   falls back to `ARP_SERVER_URL` environment variable or constructs from
    ///   `ARP_SERVER_PORT` (default: http://localhost:17004)
    /// * `arp_streaming_id` - Optional streaming session identifier. If not provided,
    ///   falls back to `ARP_STREAMING_ID` environment variable (default: "unknown")
    ///
    /// # Returns
    ///
    /// A new `PermissionManager` instance with an optimized HTTP client configured
    /// with appropriate timeouts for ARP server communication.
    pub fn new(arp_server_url: Option<String>, arp_streaming_id: Option<String>) -> Self {
        // Get configuration from parameters or environment variables
        let server_url = arp_server_url
            .or_else(|| std::env::var("ARP_SERVER_URL").ok())
            .unwrap_or_else(|| {
                let port = std::env::var("ARP_SERVER_PORT")
                    .unwrap_or_else(|_| DEFAULT_ARP_PORT.to_string());
                format!("http://localhost:{}", port)
            });

        let streaming_id = arp_streaming_id
            .or_else(|| std::env::var("ARP_STREAMING_ID").ok())
            .unwrap_or_else(|| DEFAULT_STREAMING_ID.to_string());

        Self {
            arp_server_url: server_url,
            arp_streaming_id: streaming_id,
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .connect_timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            tool_router: Self::tool_router(),
        }
    }

    /// Create a standardized error response
    fn create_error_response(message: String) -> CallToolResult {
        let deny_response = ApprovalResponse {
            behavior: "deny".to_string(),
            updated_input: None,
            message: Some(message),
        };
        CallToolResult::success(vec![Content::text(
            serde_json::to_string(&deny_response).unwrap(),
        )])
    }

    /// Create a timeout response
    fn create_timeout_response() -> CallToolResult {
        Self::create_error_response(
            "Permission request timed out after 1 hour - user did not respond".to_string(),
        )
    }

    /// Send notification to ARP server
    async fn send_notification(
        &self,
        tool_name: &str,
        input: &serde_json::Value,
    ) -> Result<String, String> {
        let notification_url = format!("{}/api/permissions/notify", self.arp_server_url);
        let request_body = PermissionNotificationRequest {
            tool_name: tool_name.to_string(),
            tool_input: input.clone(),
            streaming_id: self.arp_streaming_id.clone(),
        };

        let response = self
            .http_client
            .post(&notification_url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Failed to notify ARP server: {}", e))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Failed to notify ARP server: {}", error_text));
        }

        let notification_data: PermissionNotificationResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse notification response: {}", e))?;

        Ok(notification_data.id)
    }

    /// Poll for permission status
    async fn poll_permission_status(
        &self,
        permission_id: &str,
        tool_name: &str,
        original_input: &serde_json::Value,
    ) -> Result<CallToolResult, McpError> {
        let start_time = std::time::Instant::now();

        loop {
            // Check timeout
            if start_time.elapsed() > DEFAULT_TIMEOUT {
                tracing::warn!(
                    "Permission request timed out: tool_name={}, id={}",
                    tool_name,
                    permission_id
                );
                return Ok(Self::create_timeout_response());
            }

            // Poll for pending permissions first
            if let Some(_permission) = self
                .fetch_permission_status(permission_id, "pending")
                .await?
            {
                // Still pending, continue polling
                tokio::time::sleep(DEFAULT_POLL_INTERVAL).await;
                continue;
            }

            // Permission has been processed, fetch from all permissions
            if let Some(permission) = self.fetch_permission_status(permission_id, "").await? {
                return Ok(self.handle_permission_result(permission, tool_name, original_input));
            }

            // Wait before next poll
            tokio::time::sleep(DEFAULT_POLL_INTERVAL).await;
        }
    }

    /// Fetch permission status from ARP server
    async fn fetch_permission_status(
        &self,
        permission_id: &str,
        status_filter: &str,
    ) -> Result<Option<Permission>, McpError> {
        let mut url = format!(
            "{}/api/permissions?streamingId={}",
            self.arp_server_url, self.arp_streaming_id
        );
        if !status_filter.is_empty() {
            url.push_str(&format!("&status={}", status_filter));
        }

        let Ok(response) = self.http_client.get(&url).send().await else {
            return Ok(None);
        };
        if !response.status().is_success() {
            return Ok(None);
        }

        let Ok(permissions_data) = response.json::<PermissionsResponse>().await else {
            return Ok(None);
        };
        Ok(permissions_data
            .permissions
            .into_iter()
            .find(|p| p.id == permission_id))
    }

    /// Handle permission result and create appropriate response
    fn handle_permission_result(
        &self,
        permission: Permission,
        tool_name: &str,
        original_input: &serde_json::Value,
    ) -> CallToolResult {
        match permission.status {
            PermissionStatus::Approved => {
                tracing::debug!(
                    "Permission approved: tool_name={}, id={}",
                    tool_name,
                    permission.id
                );
                let response = ApprovalResponse {
                    behavior: "allow".to_string(),
                    updated_input: Some(
                        permission
                            .modified_input
                            .unwrap_or_else(|| original_input.clone()),
                    ),
                    message: None,
                };
                CallToolResult::success(vec![Content::text(
                    serde_json::to_string(&response).unwrap(),
                )])
            }
            PermissionStatus::Denied => {
                tracing::debug!(
                    "Permission denied: tool_name={}, id={}",
                    tool_name,
                    permission.id
                );
                let msg = permission.deny_reason.unwrap_or_else(||
                    "The user doesn't want to proceed with this tool use. The tool use was rejected. STOP what you are doing and wait for the user to tell you how to proceed.".to_string()
                );
                Self::create_error_response(msg)
            }
            PermissionStatus::Pending => {
                Self::create_error_response("Permission is still pending".to_string())
            }
        }
    }

    /// Request approval for tool usage from ARP
    #[tool(description = "Request approval for tool usage from ARP")]
    async fn approval_prompt(
        &self,
        Parameters(args): Parameters<ApprovalPromptArgs>,
    ) -> Result<CallToolResult, McpError> {
        tracing::debug!(
            "MCP Permission request received: tool_name={}, streaming_id={}",
            args.tool_name,
            self.arp_streaming_id
        );

        // Send permission notification to ARP server
        let permission_id = match self.send_notification(&args.tool_name, &args.input).await {
            Ok(id) => id,
            Err(error_msg) => {
                tracing::error!("{}", error_msg);
                return Ok(Self::create_error_response(format!(
                    "Permission denied due to error: {}",
                    error_msg
                )));
            }
        };

        tracing::debug!(
            "Permission request created: id={}, streaming_id={}",
            permission_id,
            self.arp_streaming_id
        );

        // Poll for permission decision
        self.poll_permission_status(&permission_id, &args.tool_name, &args.input)
            .await
    }
}

#[tool_handler]
impl ServerHandler for PermissionManager {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "This server provides permission management tools for ARP integration. \
                Tools: approval_prompt (requests approval for tool usage from ARP)."
                    .to_string(),
            ),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        if let Some(http_request_part) = context.extensions.get::<http::request::Parts>() {
            let initialize_headers = &http_request_part.headers;
            let initialize_uri = &http_request_part.uri;
            tracing::info!(?initialize_headers, %initialize_uri, "PermissionManager initialized from HTTP server");
        }
        tracing::info!(
            "PermissionManager initialized: server_url={}, streaming_id={}",
            self.arp_server_url,
            self.arp_streaming_id
        );
        Ok(self.get_info())
    }
}
