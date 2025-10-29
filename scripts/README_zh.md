# AgentX Proxy

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

AgentX Proxy 是一个基于 Rust 的高性能 TCP 代理系统，通过远程服务器实现本地服务的公网暴露，并提供连接池优化以降低延迟。同时支持命令执行模式，可与 Claude、Codex、Gemini 等多种 LLM 执行器配合使用。

## 🚀 核心特性

- **连接池技术**: 预建立连接，最大程度降低请求延迟
- **双模式支持**:
  - TCP 代理模式：透明转发本地 TCP/HTTP 服务
  - 命令模式：HTTP 路由 + 命令执行能力
- **多执行器支持**: 支持 Claude、Codex、Gemini 等 LLM 工具
- **MCP 集成**: Model Context Protocol 支持，扩展工具集成能力
- **会话管理**: 命令执行会话跟踪，支持输出缓冲和断线重连
- **Claude 深度集成**: 内置 Claude 项目与会话查询功能
- **高性能优化**: TCP_NODELAY + 256KB socket 缓冲区
- **跨平台**: 支持 Linux、macOS、Windows
- **自动生成客户端 ID**: 基于机器特征生成稳定的 UUID v5

## 💡 使用场景

### 🎯 场景一：将本地开发环境暴露到公网

**问题**: 您在内网开发 Web 应用，本地运行在 `127.0.0.1:3000`，但需要让远程团队成员或客户访问测试环境。

**解决方案**:
```bash
# 1. 启动 AgentX Server（公网服务器）
cargo run -p arps -- \
  --control-port 17001 \
  --proxy-port 17002 \
  --public-port 17003 \
  --pool-size 5  # 设置较大的连接池以支持多用户访问

# 2. 启动 AgentX Client（本地开发机）
cargo run -p arpc -- \
  --client-id my-dev-app \
  --server-addr your-server.com \
  --control-port 17001 \
  --proxy-port 17002 \
  --local-addr 127.0.0.1 \
  --local-port 3000

# 3. 分享访问地址给团队成员
# http://your-server.com:17003/?token=my-dev-app
```

**效果**: 任何人都可以通过 `http://your-server.com:17003/?token=my-dev-app` 访问您的本地开发环境。

### 🎯 场景二：远程使用 Claude Code 进行本地编程

**问题**: 您希望远程使用 Claude Code 等 AI 编程工具，但需要访问本地开发环境和文件。

**解决方案**:
```bash
# 1. 启动 AgentX Client（本地开发机）
cargo run -p arpc -- \
  --client-id my-programming-env \
  --server-addr your-server.com \
  --command-mode \
  --enable-mcp \
  --mcp-port 9021

# 2. 远程访问 HTTP API 进行命令执行
curl -X POST http://your-server.com:17003/?token=my-programming-env/sessions \
  -H "Content-Type: application/json" \
  -d '{
    "executor": "claude",
    "prompt": "帮我分析这个项目的代码结构",
    "project_path": "/workspace"
  }'
```

**效果**: 远程用户可以通过 HTTP API 启动 Claude 会话，在您的本地开发环境中执行命令、读取文件、运行脚本。

### 🎯 场景三：多开发者共享开发环境

**问题**: 团队成员需要共享一个集中的开发环境进行协作开发。

**解决方案**:
```bash
# 启动共享开发环境
cargo run -p arpc -- \
  --client-id shared-dev-env \
  --server-addr shared-server.com \
  --command-mode \
  --enable-mcp

# 团队成员通过不同 token 访问同一环境
# http://shared-server.com:17003/?token=alice
# http://shared-server.com:17003/?token=bob
```

### 🎯 场景四：访问内网数据库和服务

**问题**: 需要远程访问运行在企业内网的数据库、API 服务或其他应用。

**解决方案**:
```bash
# 暴露内网 PostgreSQL 数据库
cargo run -p arpc -- \
  --client-id postgres-db \
  --server-addr your-vps.com \
  --control-port 17001 \
  --proxy-port 17002 \
  --local-addr 127.0.0.1 \
  --local-port 5432

# 远程连接数据库
psql -h your-vps.com -p 17003 -d mydb "?token=postgres-db"
```

### 🎯 场景五：演示和临时环境分享

**问题**: 需要向客户或同事演示正在开发的应用，需要一个临时可访问的公网地址。

**解决方案**:
```bash
# 启动演示环境
cargo run -p arpc -- \
  --client-id demo-app-$(date +%s) \
  --server-addr demo-server.com \
  --local-addr 127.0.0.1 \
  --local-port 8080

# 生成分享链接
echo "演示地址：http://demo-server.com:17003/?token=$(grep client-id ~/.arp-client/config)"
```

## 📦 安装与构建

### 环境要求

- Rust 1.70+ ([安装 Rust](https://www.rust-lang.org/tools/install))
- Cargo（随 Rust 一起安装）

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/yourusername/agentx-proxy.git
cd agentx-proxy

# 构建所有组件
cargo build

# 构建发布版本（优化）
cargo build --release

# 构建特定组件
cargo build -p arps
cargo build -p arpc

# 运行测试
cargo test
```

## 🎯 快速开始

### 第一步：启动服务器（arps）

在公网服务器上运行：

```bash
cargo run -p arps -- \
  --control-port 17001 \
  --proxy-port 17002 \
  --public-port 17003 \
  --pool-size 1
```

服务器将监听：
- **控制端口 (17001)**: 客户端注册和控制命令
- **代理端口 (17002)**: 客户端代理连接
- **公网端口 (17003)**: 公网访问入口

### 第二步：启动客户端（arpc）

#### 模式一：TCP 代理模式（转发本地服务）

```bash
cargo run -p arpc -- \
  --client-id my-service \
  --server-addr 127.0.0.1 \
  --control-port 17001 \
  --proxy-port 17002 \
  --local-addr 127.0.0.1 \
  --local-port 3000
```

将本地 3000 端口暴露到公网。

#### 模式二：命令模式（HTTP 路由）

```bash
cargo run -p arpc -- \
  --client-id my-service \
  --command-mode \
  --enable-mcp \
  --mcp-port 9021
```

启用 HTTP API 和 MCP 支持。

### 第三步：访问服务

通过 token 访问服务：
```
http://服务器IP:17003/?token=my-service
```

## 📋 系统架构

系统由三个核心组件构成：

### 1. **arps** - 服务器组件
负责客户端注册和公网连接路由，运行三个端口：
- **控制端口**: 客户端注册和控制命令
- **代理端口**: 客户端代理连接
- **公网端口**: 公网访问入口

### 2. **arpc** - 客户端组件
连接 arps 并暴露本地服务，支持两种模式：
- **TCP 代理模式**: 透明 TCP 转发
- **命令模式**: HTTP 路由和命令执行 API

### 3. **common** - 共享协议库
协议定义、HTTP 解析工具和通用类型

## 🔌 工作原理

### 连接建立流程

1. **注册**: 客户端连接到控制端口，发送 `Register` 命令（包含 `client_id`）
2. **连接池维护**: 服务器定期请求代理连接以维持连接池
3. **公网请求**: 当公网连接到达时（携带 `?token=<client_id>`）：
   - 服务器优先检查连接池（快速路径）
   - 连接池为空时发送 `RequestNewProxyConn` 命令（慢速路径）
4. **代理连接**: 客户端连接到代理端口，发送 `NewProxyConn` 通知
5. **流合并**: 服务器将公网连接与代理连接配对，实现双向数据传输

### 协议命令

命令采用 JSON 编码 + 4 字节大端序长度前缀：

- `Register { client_id }` - 客户端注册
- `RegisterResult { success, error }` - 注册响应
- `RequestNewProxyConn { proxy_conn_id }` - 请求新代理连接
- `NewProxyConn { proxy_conn_id, client_id }` - 代理连接就绪通知

## 📚 API 参考

### 命令模式 API 端点

#### 会话管理
- `POST /api/sessions` - 创建新命令执行会话
- `GET /api/sessions/{session_id}` - 获取会话详情或重连
- `DELETE /api/sessions/{session_id}` - 取消/删除会话
- `POST /api/sessions/{session_id}/cancel` - 取消执行但保留历史

#### Claude 集成
- `GET /api/claude/projects` - 列出 Claude 项目
- `GET /api/claude/projects/working-directories` - 获取工作目录
- `GET /api/claude/projects/{project_id}/sessions` - 获取项目会话
- `GET /api/claude/sessions` - 列出所有 Claude 会话
- `GET /api/claude/sessions/{session_id}` - 加载会话消息
- `DELETE /api/claude/sessions/{session_id}` - 删除会话

#### 代理转发
- `POST /proxy` - TCP 代理转发

## 🔧 配置选项

### 服务器（arps）选项

| 选项 | 描述 | 默认值 |
|------|------|--------|
| `--control-port` | 客户端注册端口 | 17001 |
| `--proxy-port` | 代理连接端口 | 17002 |
| `--public-port` | 公网访问端口 | 17003 |
| `--pool-size` | 每客户端连接池大小 | 1 |

### 客户端（arpc）选项

| 选项 | 描述 | 默认值 |
|------|------|--------|
| `--client-id` | 唯一客户端标识符 | 自动生成 |
| `--server-addr` | 服务器地址 | 127.0.0.1 |
| `--control-port` | 服务器控制端口 | 17001 |
| `--proxy-port` | 服务器代理端口 | 17002 |
| `--local-addr` | 本地服务地址 | 127.0.0.1 |
| `--local-port` | 本地服务端口 | 必填 |
| `--command-mode` | 启用 HTTP 命令模式 | 禁用 |
| `--enable-mcp` | 启用 MCP 服务器 | 禁用 |
| `--mcp-port` | MCP 服务器端口 | 9021 |

### 环境变量

```bash
# 启用调试日志
RUST_LOG=debug cargo run -p arps

# 信息级别日志
RUST_LOG=info cargo run -p arpc
```

## 🔌 TCP 性能优化

服务器应用多种 TCP 优化技术：

- **TCP_NODELAY**: 启用以降低延迟
- **Socket 缓冲区**: 256KB 接收和发送缓冲区（`SO_RCVBUF`, `SO_SNDBUF`）
- **高效流合并**: 使用 `tokio::io::copy_bidirectional` 实现双向数据拷贝
- **连接池**: 通过预建立连接最小化延迟
- **后台池维护**: 每 5 秒补充连接以维持目标大小

## 📁 项目结构

```
agentx_proxy/
├── arp-server/                    # 服务器组件
│   └── src/main.rs           # 主服务器逻辑
├── arp-client/                   # 客户端组件
│   ├── src/
│   │   ├── main.rs           # 客户端入口
│   │   ├── config.rs         # 配置管理
│   │   ├── router.rs         # HTTP 路由逻辑
│   │   ├── routes.rs         # API 路由定义
│   │   ├── handlers/         # 请求处理器
│   │   ├── executor.rs       # 多执行器支持
│   │   ├── session.rs        # 会话管理
│   │   ├── mcp/              # MCP 服务器集成
│   │   └── claude.rs         # Claude 集成
│   └── Cargo.toml
├── arp-common/                   # 共享协议和工具
│   ├── src/
│   │   ├── lib.rs            # 协议定义
│   │   └── http.rs           # HTTP 解析工具
│   └── Cargo.toml
├── scripts/                  # 部署脚本
│   └── install_server.sh     # 服务器安装脚本
└── Cargo.toml                # 工作区配置
```

## 🧪 测试

协议支持 TCP 代理和 HTTP 模式测试。通过正确的 token 参数向公网端口发送请求。

测试流程示例：
```bash
# 启动服务器
cargo run -p arps -- --pool-size 1

# 启动客户端
cargo run -p arpc -- --client-id test --local-port 3000

# 测试连接
curl http://localhost:17003/?token=test
```

编写测试时，使用共享 Command 协议进行通信。TCP 代理和 HTTP 模式都可通过向公网端口发送带正确 token 的请求进行测试。

## 🛠️ 开发指南

### 添加新路由

1. 在 `arp-client/src/handlers/` 中添加处理器函数
2. 在 `arp-client/src/routes.rs` 中使用路由器注册路由
3. 处理器接收包含请求、流和路径参数的 `HandlerContext`
4. 返回自动发送的 `HttpResponse`

### 添加新执行器

1. 在 `arp-client/src/executor.rs` 的 `ExecutorKind` 枚举中添加变体
2. 实现 `build_<executor>_command()` 函数
3. 添加到执行器选项和 build_command 匹配
4. 更新 `storage_dir()` 返回适当的配置目录

### 代码质量

```bash
# 格式化代码
cargo fmt --all

# 运行 linter
cargo clippy --all-targets --all-features

# 检查编译
cargo check --workspace

# 构建发布版
cargo build --release
```

## 🔒 安全考虑

- 客户端 ID 应保密并用作 token
- 生产环境建议使用 HTTPS/WSS
- 为命令模式 API 实现身份验证
- 审查防火墙规则和端口暴露
- 监控连接日志以发现可疑活动

## 📊 性能指标

- **低延迟**: 通过连接池实现亚毫秒级连接建立
- **高吞吐量**: 256KB 缓冲区和 TCP_NODELAY 优化
- **可扩展**: 连接池降低多客户端开销
- **内存高效**: 无锁并发数据结构（DashMap、SegQueue）

## 🤝 贡献

欢迎贡献！请遵循以下步骤：

1. Fork 仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送分支 (`git push origin feature/amazing-feature`)
5. 打开 Pull Request

### 开发指南

- 遵循 Rust 标准格式化 (`cargo fmt`)
- 提交 PR 前运行测试 (`cargo test`)
- 为新功能添加测试
- 根据需要更新文档
- 使用有意义的提交消息

## 📝 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

- [Tokio](https://tokio.rs/) - 异步运行时
- [Clap](https://clap.rs/) - 命令行参数解析
- [Serde](https://serde.rs/) - 序列化
- [tracing](https://github.com/tokio-rs/tracing) - 结构化日志

## 🐛 故障排除

### 连接问题

- 检查防火墙设置（端口 17001-17003）
- 验证服务器是否运行并可访问
- 确保 client_id 与 token 参数匹配
- 使用 `RUST_LOG=debug` 查看日志

### 性能问题

- 增加 `--pool-size` 以获得更好的连接池
- 如需要可调整源码中的 TCP 缓冲区大小
- 监控服务器资源（CPU、内存、网络）
- 考虑使用多个服务器进行水平扩展

### 构建问题

```bash
# 清理构建
cargo clean
cargo build --release

# 更新依赖
cargo update

# 检查过时依赖
cargo outdated
```

## 📦 Cargo 特性

工作区依赖：
- `tokio` - 带完整特性的异步运行时
- `serde` - 带有 derive 特性的序列化
- `clap` - 命令行解析 v4
- `tracing` - 结构化日志
- `uuid` - UUID 生成

客户端特定特性：
- `hyper-util` - HTTP 服务器工具
- `rmcp` - Model Context Protocol 服务器
- `reqwest` - 带 JSON 和 TLS 支持的 HTTP 客户端

## 🔗 相关项目

- [Tokio](https://github.com/tokio-rs/tokio) - 异步运行时
- [Hyper](https://github.com/hyperium/hyper) - HTTP 库
- [Clap](https://github.com/clap-rs/clap) - 命令行解析器

## 📌 客户端 ID 生成

如果没有提供 client_id，arpc 会使用 UUID v5 基于机器特征生成稳定的机器特定 ID，熵来源包括：
- 主机名
- 机器 ID（Linux 上为 `/etc/machine-id`，macOS 上为 `/etc/hostid`）
- 用户名
- 操作系统和架构
- 发行版信息（仅 Linux）

## 🔗 实际应用示例

### 示例一：全栈 Web 应用开发

```bash
# 前端开发服务器（React/Vue）
cargo run -p arpc -- \
  --client-id frontend-dev \
  --server-addr your-server.com \
  --local-port 5173  # Vite 默认端口

# 后端 API 服务
cargo run -p arpc -- \
  --client-id backend-api \
  --server-addr your-server.com \
  --local-port 8000

# 分享给团队
echo "前端：http://your-server.com:17003/?token=frontend-dev"
echo "后端：http://your-server.com:17003/?token=backend-api"
```

### 示例二：数据库远程访问

```bash
# 暴露本地 MySQL
cargo run -p arpc -- \
  --client-id mysql-db \
  --server-addr your-server.com \
  --local-port 3306

# 远程连接
mysql -h your-server.com -P 17003 -u root -p "?token=mysql-db"
```

### 示例三：CI/CD 测试环境

```bash
# 暴露测试服务器
cargo run -p arpc -- \
  --client-id ci-test \
  --server-addr ci-server.com \
  --local-port 8080

# GitHub Actions 中自动测试
curl -X POST http://ci-server.com:17003/?token=ci-test/api/deploy
```

---

**注意**: 这是一个开源项目。如有问题和功能请求，请访问 [GitHub Issues](https://github.com/yourusername/agentx-proxy/issues) 页面。
