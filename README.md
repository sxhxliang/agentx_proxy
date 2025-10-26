# AgentX Proxy

## 项目简介

AgentX Proxy 是一个基于 Rust 的远程代理与Claude Code指令执行工具，用于将本地智能体或开发者工作流暴露给远程使用者。项目包含服务端 `agents`、边缘客户端 `agentc` 与共享协议库 `common` 三个部分，利用长连接和自定义命令协议实现：

- 公网入口统一接入与路由。
- 客户端连接池复用，降低建立隧道的开销。
- “命令模式”下的 HTTP API，用于启动、查询和管理本地 LLM/自动化会话（支持 Claude/Codex/Gemini 等执行器）。

## 架构总览

| 组件 | 作用 |
| --- | --- |
| `agents` | 部署在公网的代理服务。监听控制端口、代理端口和公网端口，负责客户端注册、连接池维护以及将公网请求分发给对应的客户端。 |
| `agentc` | 运行在本地或边缘节点的客户端。常规模式下转发本地 TCP/HTTP 服务；命令模式下接管请求，由内置路由器执行命令会话和 Claude 历史查询等操作。 |
| `common` | 定义双端共享的命令协议（长度前缀 + JSON）以及轻量 HTTP 抽象，简化命令读写与请求解析。 |

### 工作流程

1. `agentc` 通过控制端口与 `agents` 建立连接并注册自身 `client_id`。
2. `agents` 为每个活跃客户端维护一个 `proxy` 连接池。当公网有请求到达时，优先复用现有连接；必要时向客户端请求新的代理隧道。
3. 客户端在收到 `RequestNewProxyConn` 后连接代理端口，完成握手后：
   - **转发模式**：将公网请求映射到本地 `local_addr:local_port`，实现安全的远程访问。
   - **命令模式**：由内置 HTTP 路由器处理请求，启动或查询命令会话、读取 Claude 历史等。
4. `agents` 在公网端口同时支持原始 TCP 透传和 HTTP 请求解析，允许根据首包内容自动选择处理方式。

## 主要特性

- **连接池与预热**：服务端按需为客户端补齐代理连接，降低高并发场景下的排队与握手延迟。
- **命令会话管理**：客户端维护 `SessionManager`，支持启动、取消、续接命令执行，并缓冲标准输出供断线重连。
- **Claude 会话工具**：读取 `~/.claude` 目录，提供项目、会话与工作目录的 HTTP 查询能力。
- **可插拔执行器**：通过 `executor::ExecutorKind` 支持 Claude、Codex 以及预留的 Gemini，自动拼装命令行参数。

## 快速开始

### 环境要求

- Rust 1.74+（2021 edition）
- 目标主机可访问公网端口 17001/17002/17003，或自行指定
- 可选：本地已安装 Claude CLI / Codex CLI 等执行器

### 构建与检查

```bash
cargo fmt --all
cargo clippy --all-targets --all-features
cargo check --workspace
cargo build --release
```

### 启动 agents 服务端

```bash
cargo run -p agents -- \
  --control-port 17001 \
  --proxy-port 17002 \
  --public-port 17003 \
  --pool-size 2
```

- `control-port`：客户端注册与控制命令。
- `proxy-port`：代理隧道建立，传输真实业务流量。
- `public-port`：对外暴露的公网入口；支持 HTTP、WebSocket 以及原始 TCP。
- `pool-size`：每个客户端预热的代理连接数。

生产环境可先执行 `cargo build --release`，再运行 `scripts/install_server.sh` 将二进制安装到 `/usr/local/bin/agents` 并注册 systemd 服务。可通过环境变量 `CONTROL_PORT`、`PROXY_PORT`、`PUBLIC_PORT` 覆盖默认值。

### 启动 agentc 客户端


```bash
cargo run -p agentc -- \
  --server-addr 你的服务器地址 \
  --command-mode \
  --command-path "/usr/local/bin/claude" \ # 可选 会自动寻找
```

- `--command-mode`：公网请求将由内置路由器处理，不再转发到本地服务。

`agentc` 启动时会自动生成或读取 `client_id`，并在控制通道完成注册握手，填进前端页面中。

## 命令模式 API 速览

命令模式下，公网请求可直接访问以下 HTTP 端点（默认返回 JSON）：

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `POST /sessions` | 创建新的命令执行会话，支持选择执行器、模型、提示语、项目路径等。 |
| `GET /sessions/{session_id}` | 查看会话状态与输出，或在连接中断后重新订阅输出。 |
| `DELETE /sessions/{session_id}` | 终止并删除指定会话。 |
| `POST /sessions/{session_id}/cancel` | 向正在运行的会话发送取消信号，仅结束执行但保留历史。 |
| `GET /claude/projects` | 枚举本地 Claude 项目与最近会话。 |
| `GET /claude/projects/{project_id}/sessions` | 查看指定项目下的 Claude 会话列表。 |
| `GET /claude/sessions` | 按分页检索 Claude 会话，可通过 `limit`、`offset`、`projectPath` 过滤。 |
| `GET /claude/sessions/{session_id}` | 加载单个 Claude 会话的消息历史。 |
| `DELETE /claude/sessions/{session_id}` | 删除本地存储的 Claude 会话文件。 |

除上述接口外，`routes.rs` 中还可以自定义命令路由以满足特定业务需求。

## 目录结构

```
agentx_proxy/
├── agents/        # 公网代理服务
│   └── src/main.rs
├── agentc/        # 边缘客户端与命令执行入口
│   └── src/
│       ├── main.rs
│       ├── config.rs
│       ├── router.rs
│       ├── routes.rs
│       └── handlers/
├── common/        # 协议与 HTTP 工具
│   └── src/
│       ├── lib.rs
│       └── http.rs
├── scripts/       # 运维脚本（systemd 安装）
│   └── install_server.sh
└── Cargo.toml     # Workspace 配置
```

## 开发与测试建议

- 功能开发前运行 `cargo check --workspace` 以快速验证编译。
- 保持格式与 lint：`cargo fmt --all`、`cargo clippy --all-targets --all-features`。
- 执行单元与集成测试：`cargo test --workspace` 或按 crate 指定 `-p agentc`。
- 新增配置项时同步更新 `agentc/src/config.rs` 与相关脚本/文档，确保部署一致。
- 涉及端口或部署行为更改时，请同步维护 `scripts/`、`docs/DEPLOYMENT.md` 以及 `nginx/agents.conf`（若适用）。

## 后续方向

- 扩展命令执行器（如完善 Gemini 支持或添加自定义工具链）。
- 为 `agents` 与 `agentc` 增加更多端到端测试，覆盖连接池与超时清理逻辑。
- 对 MCP 模块补充文档与示例，便于 IDE 集成。

欢迎提交 Issue 或 Pull Request，一起完善 AgentX Proxy。
