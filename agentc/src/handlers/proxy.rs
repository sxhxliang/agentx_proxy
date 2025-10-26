use crate::handlers::HandlerState;
use crate::router::HandlerContext;
use anyhow::Result;
use common::http::HttpResponse;
use common::join_streams;
use tokio::net::TcpStream;
use tracing::info;

/// Handle TCP proxy requests
pub async fn handle_proxy(ctx: HandlerContext, state: HandlerState) -> Result<HttpResponse> {
    let proxy_conn_id = &ctx.proxy_conn_id;
    let config = &state.config;

    // Connect to local service
    let local_stream = TcpStream::connect(config.local_service_addr()).await?;
    info!(
        "('{}') Connected to local service at {}.",
        proxy_conn_id,
        config.local_service_addr()
    );

    // Join streams (proxy <-> local service)
    info!("('{}') Joining streams...", proxy_conn_id);
    join_streams(ctx.stream, local_stream).await?;
    info!("('{}') Streams joined and finished.", proxy_conn_id);

    // Return a dummy response (stream already handled)
    Ok(HttpResponse::ok())
}
