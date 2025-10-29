use anyhow::{anyhow, Result};
use clap::Parser;
use common::http::{HttpRequest, HttpResponse};
use common::{join_streams, read_command, write_command, Command};
use crossbeam::queue::SegQueue;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn, Level};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = 17001)]
    control_port: u16,

    #[arg(long, default_value_t = 17002)]
    proxy_port: u16,

    #[arg(long, default_value_t = 17003)]
    public_port: u16,

    #[arg(long, default_value_t = 3)]
    pool_size: usize,
}

struct ClientInfo {
    writer: Arc<Mutex<OwnedWriteHalf>>,
    pool: Arc<SegQueue<TcpStream>>,
}

// Use DashMap for lock-free concurrent access to active clients
type ActiveClients = Arc<DashMap<String, Arc<ClientInfo>>>;

// Pending connection with timestamp for timeout tracking
struct PendingConnection {
    stream: TcpStream,
    timestamp: std::time::Instant,
    http_request: Option<HttpRequest>,
}

// Use DashMap for lock-free concurrent access to pending connections
type PendingConnectionsMap = Arc<DashMap<String, PendingConnection>>;

// Global counter for fast ID generation
static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

fn generate_id() -> String {
    let id = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{:x}", id)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let active_clients: ActiveClients = Arc::new(DashMap::new());
    let pending_connections: PendingConnectionsMap = Arc::new(DashMap::new());

    let control_listener = TcpListener::bind(format!("0.0.0.0:{}", args.control_port)).await?;
    let proxy_listener = TcpListener::bind(format!("0.0.0.0:{}", args.proxy_port)).await?;
    let public_listener = TcpListener::bind(format!("0.0.0.0:{}", args.public_port)).await?;

    info!(
        "arps listening on ports: Control={}, Proxy={}, Public={}, Pool Size={}",
        args.control_port, args.proxy_port, args.public_port, args.pool_size
    );

    // Spawn background task to maintain connection pools
    let pool_maintainer_clients = active_clients.clone();
    let target_pool_size = args.pool_size;
    tokio::spawn(async move {
        maintain_connection_pools(pool_maintainer_clients, target_pool_size).await;
    });

    // Spawn background task to cleanup expired pending connections
    let cleanup_pending = pending_connections.clone();
    tokio::spawn(async move {
        cleanup_expired_connections(cleanup_pending).await;
    });

    let server_logic = tokio::select! {
        res = handle_control_connections(control_listener, active_clients.clone()) => res,
        res = handle_proxy_connections(proxy_listener, pending_connections.clone(), active_clients.clone()) => res,
        res = handle_public_connections(public_listener, active_clients.clone(), pending_connections.clone()) => res,
    };

    if let Err(e) = server_logic {
        error!("Server error: {}", e);
    }

    Ok(())
}

/// Optimizes TCP socket settings for low latency and high throughput
fn tune_tcp_socket(stream: &TcpStream) -> Result<()> {
    use std::os::fd::AsRawFd;

    stream.set_nodelay(true)?;

    let fd = stream.as_raw_fd();
    unsafe {
        let buf_size: libc::c_int = 262144; // 256KB
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_RCVBUF,
            &buf_size as *const _ as *const libc::c_void,
            std::mem::size_of::<libc::c_int>() as libc::socklen_t,
        );
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_SNDBUF,
            &buf_size as *const _ as *const libc::c_void,
            std::mem::size_of::<libc::c_int>() as libc::socklen_t,
        );
    }
    Ok(())
}

async fn handle_control_connections(
    listener: TcpListener,
    active_clients: ActiveClients,
) -> Result<()> {
    loop {
        let (stream, addr) = listener.accept().await?;
        info!("New control connection from: {}", addr);

        // Tune TCP socket for control connection
        if let Err(e) = tune_tcp_socket(&stream) {
            warn!("Failed to tune control socket for {}: {}", addr, e);
        }

        let active_clients_clone = active_clients.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_single_client(stream, active_clients_clone).await {
                error!("Error handling client {}: {}", addr, e);
            }
        });
    }
}

async fn handle_single_client(stream: TcpStream, active_clients: ActiveClients) -> Result<()> {
    let (mut reader, writer) = stream.into_split();
    let writer = Arc::new(Mutex::new(writer));

    let client_id = if let Command::Register { client_id: id } = read_command(&mut reader).await? {
        info!("Registration attempt for client_id: {}", id);

        // Remove old registration if exists (allow reconnection)
        if let Some((_, old_info)) = active_clients.remove(&id) {
            warn!("Client ID {} was already registered, replacing with new connection.", id);
            // Clear old pool connections
            while old_info.pool.pop().is_some() {}
        }

        active_clients.insert(
            id.clone(),
            Arc::new(ClientInfo {
                writer: writer.clone(),
                pool: Arc::new(SegQueue::new()),
            }),
        );
        let _ = write_command(
            &mut *writer.lock().await,
            &Command::RegisterResult {
                success: true,
                error: None,
            },
        )
        .await;
        info!("Client {} registered successfully.", id);
        id
    } else {
        return Err(anyhow!("First command was not Register"));
    };

    // Keep reading from the control channel, but we don't expect more commands.
    // The main purpose is to detect when the client disconnects.
    loop {
        if reader.read_u8().await.is_err() {
            warn!("Client {} disconnected.", client_id);
            if let Some((_, old_info)) = active_clients.remove(&client_id) {
                // Clear pool connections when client disconnects
                while old_info.pool.pop().is_some() {}
            }
            break;
        }
    }

    Ok(())
}

async fn handle_proxy_connections(
    listener: TcpListener,
    pending_connections: PendingConnectionsMap,
    active_clients: ActiveClients,
) -> Result<()> {
    loop {
        let (mut proxy_stream, addr) = listener.accept().await?;
        info!("New proxy connection from: {}", addr);

        // Tune TCP socket for proxy connection (high throughput)
        if let Err(e) = tune_tcp_socket(&proxy_stream) {
            warn!("Failed to tune proxy socket for {}: {}", addr, e);
        }

        let pending_clone = pending_connections.clone();
        let clients_clone = active_clients.clone();

        tokio::spawn(async move {
            if let Ok(Command::NewProxyConn {
                proxy_conn_id,
                client_id,
            }) = read_command(&mut proxy_stream).await
            {
                info!(
                    "Received proxy conn notification for id: {} from client: {}",
                    proxy_conn_id, client_id
                );
                if let Some((_, pending_conn)) = pending_clone.remove(&proxy_conn_id) {
                    let user_stream = pending_conn.stream;
                    let http_request = pending_conn.http_request;
                    info!(
                        "Pairing user stream with proxy stream for id: {}",
                        proxy_conn_id
                    );
                    tokio::spawn(async move {
                        // If there's a parsed HTTP request, reconstruct it first
                        if let Some(request) = http_request {
                            if let Err(e) = write_http_request(&mut proxy_stream, &request).await {
                                error!("Failed to write HTTP request to proxy stream: {}", e);
                                return;
                            }
                        }

                        // Now join the streams
                        if let Err(e) = join_streams(user_stream, proxy_stream).await {
                            error!("Error joining streams: {}", e);
                        }
                        info!("Streams for {} joined and finished.", proxy_conn_id);
                    });
                } else {
                    // No pending request - this is for the pool
                    info!(
                        "No pending request for {}, adding to client {} pool",
                        proxy_conn_id, client_id
                    );
                    if let Some(client_info) = clients_clone.get(&client_id) {
                        client_info.pool.push(proxy_stream);
                        info!("Added connection to pool for client {}", client_id);
                    } else {
                        warn!("Client {} not found for pool connection", client_id);
                    }
                }
            } else {
                error!("Failed to read NewProxyConn command from {}", addr);
            }
        });
    }
}

async fn handle_public_connections(
    listener: TcpListener,
    active_clients: ActiveClients,
    pending_connections: PendingConnectionsMap,
) -> Result<()> {
    loop {
        let (user_stream, addr) = listener.accept().await?;
        info!("New public connection from: {}", addr);

        // Tune TCP socket for public connection (low latency critical)
        if let Err(e) = tune_tcp_socket(&user_stream) {
            warn!("Failed to tune public socket for {}: {}", addr, e);
        }

        let active_clients_clone = active_clients.clone();
        let pending_connections_clone = pending_connections.clone();

        tokio::spawn(async move {
            if let Err(e) = route_public_connection(
                user_stream,
                active_clients_clone,
                pending_connections_clone,
            )
            .await
            {
                error!("Failed to route public connection from {}: {}", addr, e);
            }
        });
    }
}

/// Reconstruct HTTP request and write it to a stream
async fn write_http_request(stream: &mut TcpStream, request: &HttpRequest) -> Result<()> {
    use tokio::io::AsyncWriteExt;

    // Reconstruct request line with query parameters
    let query_string = if request.query_params.is_empty() {
        String::new()
    } else {
        let params: Vec<String> = request
            .query_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect();
        format!("?{}", params.join("&"))
    };

    let request_line = format!(
        "{} {}{} HTTP/1.1\r\n",
        request.method.as_str(),
        request.path,
        query_string
    );
    stream.write_all(request_line.as_bytes()).await?;

    // Write headers
    for (key, value) in &request.headers {
        stream
            .write_all(format!("{}: {}\r\n", key, value).as_bytes())
            .await?;
    }

    // End of headers
    stream.write_all(b"\r\n").await?;

    // Write body
    if !request.body.is_empty() {
        stream.write_all(&request.body).await?;
    }

    stream.flush().await?;
    Ok(())
}

async fn route_public_connection(
    mut user_stream: TcpStream,
    active_clients: ActiveClients,
    pending_connections: PendingConnectionsMap,
) -> Result<()> {
    // Try to parse as HTTP request to extract token
    let proxy_conn_id_for_parsing = generate_id();
    let http_request = match HttpRequest::parse(&mut user_stream, &proxy_conn_id_for_parsing).await
    {
        Ok(req) => Some(req),
        Err(e) => {
            warn!("Failed to parse HTTP request: {}, treating as raw TCP", e);
            None
        }
    };

    // Phase 1: Determine which client to route to based on token (if present)
    if active_clients.is_empty() {
        warn!("No active clients available to handle new public connection.");

        // If we parsed HTTP, send 503 Service Unavailable
        if http_request.is_some() {
            let _ = HttpResponse::new(503)
                .text("No active clients available")
                .send(&mut user_stream)
                .await;
        }

        return Err(anyhow!("No active clients"));
    }

    // Check if token parameter exists in HTTP request
    let token = match http_request
        .as_ref()
        .and_then(|req| req.query_param("token"))
    {
        Some(t) => t,
        None => {
            if http_request.is_some() {
                let _ = HttpResponse::not_found()
                    .text("Client Token not found")
                    .send(&mut user_stream)
                    .await;
            }
            return Err(anyhow!("Client Token not found"));
        }
    };

    // Token-based routing
    info!("Token-based routing: looking for client_id '{}'", token);

    let client_info = match active_clients.get(token) {
        Some(info) => info,
        None => {
            warn!("Client '{}' not found for token", token);
            if http_request.is_some() {
                let _ = HttpResponse::not_found()
                    .text(format!("Client '{}' not found", token))
                    .send(&mut user_stream)
                    .await;
            }
            return Err(anyhow!("Client '{}' not found", token));
        }
    };

    info!("Found client '{}' matching token", token);

    // Phase 2: Try to get connection from pool first (fast path)
    if let Some(mut proxy_stream) = client_info.pool.pop() {
        info!("Using pooled connection (fast path)");

        // If we parsed HTTP, we need to reconstruct and send the request
        if let Some(request) = http_request {
            // Write reconstructed HTTP request to proxy stream
            if let Err(e) = write_http_request(&mut proxy_stream, &request).await {
                error!("Failed to write HTTP request to proxy stream: {}", e);
                return Err(e);
            }
        }

        // Join the streams directly
        if let Err(e) = join_streams(user_stream, proxy_stream).await {
            error!("Error joining streams from pool: {}", e);
        }

        return Ok(());
    }

    // Phase 3: Fallback to traditional proxy request (slow path)
    let proxy_conn_id = generate_id();
    let command = Command::RequestNewProxyConn {
        proxy_conn_id: proxy_conn_id.clone(),
    };

    info!(
        "Pool empty, requesting new proxy connection with id: {}",
        proxy_conn_id
    );

    // Insert into pending before sending command to avoid race condition
    let pending_conn = PendingConnection {
        stream: user_stream,
        timestamp: std::time::Instant::now(),
        http_request,
    };
    pending_connections.insert(proxy_conn_id.clone(), pending_conn);

    // Send command to client
    let mut writer = client_info.writer.lock().await;
    if let Err(e) = write_command(&mut *writer, &command).await {
        error!(
            "Failed to send RequestNewProxyConn to client: {}. Connection will timeout.",
            e
        );
        // Clean up pending connection
        pending_connections.remove(&proxy_conn_id);
        return Err(e);
    }

    info!(
        "Successfully sent RequestNewProxyConn with id {}",
        proxy_conn_id
    );
    Ok(())
}

// Background task to cleanup expired pending connections
async fn cleanup_expired_connections(pending_connections: PendingConnectionsMap) {
    let mut ticker = interval(Duration::from_secs(2));
    const TIMEOUT_SECS: u64 = 10;

    loop {
        ticker.tick().await;

        let now = std::time::Instant::now();
        let initial_count = pending_connections.len();

        // Remove expired connections
        pending_connections.retain(|id, conn| {
            let age = now.duration_since(conn.timestamp);
            if age.as_secs() > TIMEOUT_SECS {
                warn!(
                    "Removing expired pending connection {} (age: {:?})",
                    id, age
                );
                false
            } else {
                true
            }
        });

        let removed = initial_count - pending_connections.len();
        if removed > 0 {
            info!("Cleaned up {} expired pending connections", removed);
        }
    }
}

// Background task to maintain connection pools for all clients
async fn maintain_connection_pools(active_clients: ActiveClients, target_pool_size: usize) {
    let mut ticker = interval(Duration::from_secs(5));

    loop {
        ticker.tick().await;

        for entry in active_clients.iter() {
            let (client_id, client_info) = entry.pair();
            let current_size = client_info.pool.len();

            if current_size < target_pool_size {
                let needed = target_pool_size - current_size;

                info!(
                    "Client {} pool has {} connections, requesting {} more",
                    client_id, current_size, needed
                );

                // Request additional connections to fill the pool
                for _ in 0..needed {
                    let pool_conn_id = generate_id();
                    let command = Command::RequestNewProxyConn {
                        proxy_conn_id: pool_conn_id.clone(),
                    };

                    let mut writer = client_info.writer.lock().await;
                    if let Err(e) = write_command(&mut *writer, &command).await {
                        error!("Failed to request pool connection for {}: {}", client_id, e);
                        break;
                    }
                    drop(writer); // Release lock between requests
                }
            }
        }
    }
}
