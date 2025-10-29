# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

AgentX Proxy is a Rust-based TCP proxy system for exposing local services through a remote server. It consists of two main components:

1. **arps** - The server component that accepts client registrations and routes public connections
2. **arpc** - The client component that connects to arps and exposes local services

The system uses a connection pooling architecture to minimize latency for incoming requests.

## Workspace Structure

This is a Cargo workspace with three crates:

- `arp-server/` - Server binary
- `arp-client/` - Client binary with HTTP routing and command execution capabilities
- `arp-common/` - Shared protocol definitions and utilities

## Build Commands

```bash
# Build all workspace members
cargo build

# Build release binaries
cargo build --release

# Build specific crate
cargo build -p arps
cargo build -p arpc

# Run tests
cargo test

# Run with logging
RUST_LOG=info cargo run -p arps
RUST_LOG=info cargo run -p arpc
```

## Running the System

### Start the Server (arps)

```bash
cargo run -p arps -- \
  --control-port 17001 \
  --proxy-port 17002 \
  --public-port 17003 \
  --pool-size 1
```

The server listens on three ports:
- **Control port (17001)**: Client registration and control commands
- **Proxy port (17002)**: Proxy connections from clients
- **Public port (17003)**: Public-facing connections that are routed to clients

### Start the Client (arpc)

Basic TCP proxy mode:
```bash
cargo run -p arpc -- \
  --client-id <UNIQUE_ID> \
  --server-addr 127.0.0.1 \
  --control-port 17001 \
  --proxy-port 17002 \
  --local-addr 127.0.0.1 \
  --local-port 3000
```

Command mode (HTTP routing):
```bash
cargo run -p arpc -- \
  --client-id <UNIQUE_ID> \
  --command-mode \
  --enable-mcp \
  --mcp-port 9021
```

## Architecture

### Protocol Flow

1. **Registration**: Client connects to control port and sends `Register` command with client_id
2. **Pool Maintenance**: Server periodically requests proxy connections to maintain a connection pool
3. **Public Request**: When a public connection arrives with `?token=<client_id>`:
   - Server first checks the connection pool for available connections (fast path)
   - If pool is empty, sends `RequestNewProxyConn` command to client (slow path)
4. **Proxy Connection**: Client connects to proxy port and sends `NewProxyConn` notification
5. **Stream Joining**: Server pairs the public connection with the proxy connection and joins streams bidirectionally

### Connection Pooling

The server maintains a connection pool per client (configurable via `--pool-size`). This minimizes latency by having pre-established connections ready. A background task runs every 5 seconds to refill pools that fall below the target size.

### Command Protocol (arp-common/src/lib.rs)

Commands are JSON-encoded with a 4-byte big-endian length prefix:
- `Register { client_id }` - Client registration
- `RegisterResult { success, error }` - Registration response
- `RequestNewProxyConn { proxy_conn_id }` - Request new proxy connection
- `NewProxyConn { proxy_conn_id, client_id }` - Notify proxy connection ready

### HTTP Support (arp-common/src/http.rs)

The `HttpRequest` and `HttpResponse` types provide HTTP/1.1 parsing and response building with automatic CORS header injection. Query parameters are URL-decoded and stored in a HashMap.

### arpc Architecture

When `--command-mode` is enabled, arpc runs an HTTP router instead of simple TCP proxying:

- **Router** (arp-client/src/router.rs): Pattern-based HTTP routing with path parameters
- **Handlers** (arp-client/src/handlers/):
  - `session` - Command execution session management
  - `proxy` - TCP proxy forwarding
- **Executor** (arp-client/src/executor.rs): Multi-executor support (Claude, Codex, Gemini)
- **Session Manager** (arp-client/src/session.rs): Tracks running command sessions with output buffering
- **MCP Server** (arp-client/src/mcp/): Model Context Protocol server for tool integration

### API Routes (arp-client/src/routes.rs)

Session management:
- `POST /api/sessions` - Create new command execution session
- `GET /api/sessions/{session_id}` - Get session details or reconnect
- `DELETE /api/sessions/{session_id}` - Cancel/delete session
- `POST /api/sessions/{session_id}/cancel` - Cancel without deleting history

Claude integration:
- `GET /api/claude/projects` - List Claude projects
- `GET /api/claude/projects/working-directories` - Get working directories
- `GET /api/claude/projects/{project_id}/sessions` - Get project sessions
- `GET /api/claude/sessions` - List all Claude sessions
- `GET /api/claude/sessions/{session_id}` - Load session messages
- `DELETE /api/claude/sessions/{session_id}` - Delete session

## Client ID Generation

If no client_id is provided, arpc generates a stable machine-specific ID using UUID v5 with entropy from:
- Hostname
- Machine ID (`/etc/machine-id` on Linux, `/etc/hostid` on macOS)
- Username
- OS and architecture
- Distribution info (Linux only)

## TCP Optimizations (arp-server/src/main.rs:102-126)

The server applies TCP tuning to all sockets:
- `TCP_NODELAY` enabled for low latency
- 256KB socket buffers (`SO_RCVBUF`, `SO_SNDBUF`)
- Uses `tokio::io::copy_bidirectional` for efficient stream joining

## Testing

When writing tests, use the shared Command protocol for communication. Both TCP proxy and HTTP modes are testable by sending appropriate requests to the public port with the correct token parameter.

## Common Development Patterns

### Adding a new route

1. Add handler function in `arp-client/src/handlers/`
2. Register route in `arp-client/src/routes.rs` using the router
3. Handler receives `HandlerContext` with request, stream, and path parameters
4. Return `HttpResponse` which is sent automatically

### Adding a new executor

1. Add variant to `ExecutorKind` enum in `arp-client/src/executor.rs`
2. Implement `build_<executor>_command()` function
3. Add to executor options and build_command match
4. Update storage_dir() to return appropriate config directory
