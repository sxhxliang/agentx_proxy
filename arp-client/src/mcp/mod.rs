pub mod permissions;
use permissions::PermissionManager;

use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder,
    service::TowerToHyperService,
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};

/// Start the MCP server on the specified port
pub async fn start_mcp_server(port: u16) -> anyhow::Result<()> {
    let service = TowerToHyperService::new(StreamableHttpService::new(
        || Ok(PermissionManager::new(None, None)),
        LocalSessionManager::default().into(),
        Default::default(),
    ));
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tracing::info!("MCP server listening on 0.0.0.0:{}", port);

    loop {
        let io = tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("MCP server shutting down");
                break;
            },
            accept = listener.accept() => {
                TokioIo::new(accept?.0)
            }
        };
        let service = service.clone();
        tokio::spawn(async move {
            let _result = Builder::new(TokioExecutor::default())
                .serve_connection(io, service)
                .await;
        });
    }
    Ok(())
}
