mod agentx;
mod config;
mod error;
mod executor;
mod handlers;
mod mcp;
mod router;
mod routes;
mod session;

use anyhow::{anyhow, Result};
use clap::Parser;
use common::http;
use common::{read_command, write_command, Command};
use config::ClientConfig;
use handlers::HandlerState;
use router::{HandlerContext, Router};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io;
use tokio::net::TcpStream;
use tracing::{error, info, warn, Level};

#[tokio::main]
async fn main() -> Result<()> {
    let config = ClientConfig::parse();
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Validate configuration
    if let Err(e) = config.validate() {
        error!("Configuration validation failed: {}", e);
        return Err(anyhow!("Invalid configuration: {}", e));
    }

    info!("Starting agentc with client_id: {}", config.client_id);
    info!("Server address: {}", config.control_addr());
    info!("Local service: {}", config.local_service_addr());

    // Start MCP server if enabled
    if config.enable_mcp {
        let mcp_port = config.mcp_port;
        tokio::spawn(async move {
            if let Err(e) = mcp::start_mcp_server(mcp_port).await {
                error!("MCP server error: {}", e);
            }
        });
        info!("MCP server enabled on port {}", config.mcp_port);
    }

    // Create shared state
    let state = HandlerState::new(config.clone());

    // Extract Arc-wrapped config to avoid repeated cloning in the loop
    let config_arc = state.config.clone();

    // Build router and wrap in Arc to avoid repeated cloning
    let router = Arc::new(routes::build_router(state));

    let control_stream = TcpStream::connect(config.control_addr()).await?;
    info!("Connected to control port.");

    let (mut reader, mut writer) = tokio::io::split(control_stream);

    // Register the client
    let register_cmd = Command::Register {
        client_id: config.client_id.clone(),
    };
    write_command(&mut writer, &register_cmd).await?;

    // Wait for registration result
    match read_command(&mut reader).await? {
        Command::RegisterResult { success, error } => {
            if success {
                info!("Successfully registered with the server.");
            } else {
                error!("Registration failed: {}", error.unwrap_or_default());
                return Err(anyhow!("Registration failed"));
            }
        }
        _ => {
            return Err(anyhow!(
                "Received unexpected command after registration attempt."
            ));
        }
    }

    // Main loop to listen for commands from the server
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl+C signal. Shutting down gracefully...");
                break;
            }
            result = read_command(&mut reader) => {
                match result {
                    Ok(Command::RequestNewProxyConn { proxy_conn_id }) => {
                        info!("Received request for new proxy connection: {}", proxy_conn_id);
                        // Use Arc::clone for efficient reference counting instead of deep cloning
                        let config_ref = Arc::clone(&config_arc);
                        let router_ref = Arc::clone(&router);
                        tokio::spawn(async move {
                            if let Err(e) = create_proxy_connection(config_ref, router_ref, proxy_conn_id).await {
                                error!("Failed to create proxy connection: {}", e);
                            }
                        });
                    }
                    Ok(cmd) => {
                        warn!("Received unexpected command: {:?}", cmd);
                    }
                    Err(ref e) if e.downcast_ref::<io::Error>().is_some_and(|io_err| io_err.kind() == io::ErrorKind::UnexpectedEof) => {
                        error!("Control connection closed by server. Shutting down.");
                        break;
                    }
                    Err(e) => {
                        error!("Error reading from control connection: {}. Shutting down.", e);
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn create_proxy_connection(
    config: Arc<ClientConfig>,
    router: Arc<Router>,
    proxy_conn_id: String,
) -> Result<()> {
    let command_mode_enabled = config.command_mode;
    let mut proxy_stream = TcpStream::connect(config.proxy_addr()).await?;
    info!("('{}') Connected to proxy port.", proxy_conn_id);

    let notify_cmd = Command::NewProxyConn {
        proxy_conn_id: proxy_conn_id.clone(),
        client_id: config.client_id.clone(),
    };
    write_command(&mut proxy_stream, &notify_cmd).await?;
    info!(
        "('{}') Sent new proxy connection notification.",
        proxy_conn_id
    );

    if command_mode_enabled {
        handle_command_mode_connection(proxy_stream, router, proxy_conn_id).await
    } else {
        handle_tcp_proxy_connection(config, proxy_stream, proxy_conn_id).await
    }
}

async fn handle_command_mode_connection(
    mut proxy_stream: TcpStream,
    router: Arc<Router>,
    proxy_conn_id: String,
) -> Result<()> {
    info!(
        "('{}') Running in command mode (HTTP routing)",
        proxy_conn_id
    );

    match http::HttpRequest::parse(&mut proxy_stream, &proxy_conn_id).await {
        Ok(request) => {
            // Handle CORS preflight early to avoid empty responses
            if request.method == http::HttpMethod::OPTIONS {
                let stream = &mut proxy_stream;
                let _ = http::HttpResponse::new(204)
                    .header("Access-Control-Allow-Origin", "*")
                    .header(
                        "Access-Control-Allow-Methods",
                        "GET, POST, PUT, DELETE, PATCH, OPTIONS",
                    )
                    .header(
                        "Access-Control-Allow-Headers",
                        "Content-Type, Authorization",
                    )
                    .header("Access-Control-Max-Age", "86400")
                    .body(Vec::new())
                    .send(stream)
                    .await;
                info!(
                    "('{}') Responded to CORS preflight (OPTIONS)",
                    proxy_conn_id
                );
                return Ok(());
            }

            let ctx = HandlerContext {
                request,
                stream: proxy_stream,
                proxy_conn_id: proxy_conn_id.clone(),
                path_params: HashMap::new(),
            };

            match router.handle(ctx).await {
                Ok(_response) => {
                    info!("('{}') Request handled successfully", proxy_conn_id);
                }
                Err(e) => {
                    error!("('{}') Handler error: {}", proxy_conn_id, e);
                }
            }
        }
        Err(e) => {
            error!("('{}') Failed to parse HTTP request: {}", proxy_conn_id, e);
        }
    }

    Ok(())
}

async fn handle_tcp_proxy_connection(
    config: Arc<ClientConfig>,
    proxy_stream: TcpStream,
    proxy_conn_id: String,
) -> Result<()> {
    // Clone the config from Arc for HandlerState::new
    let state = HandlerState::new((*config).clone());
    let ctx = HandlerContext {
        request: http::HttpRequest {
            method: http::HttpMethod::GET,
            path: "/".to_string(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body: Vec::new(),
        },
        stream: proxy_stream,
        proxy_conn_id: proxy_conn_id.clone(),
        path_params: HashMap::new(),
    };

    match handlers::proxy::handle_proxy(ctx, state).await {
        Ok(_) => {
            info!("('{}') TCP proxy completed successfully", proxy_conn_id);
        }
        Err(e) => {
            error!("('{}') TCP proxy error: {}", proxy_conn_id, e);
        }
    }

    Ok(())
}
