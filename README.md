# ARP - AI Agent Reverse Proxy

> **随时随地访问您的内网 AI 智能体**

## 🎯 核心价值

**您的 AI 智能体运行在公司内网/家庭网络，但您需要在任何地方访问它？**

ARP（AI Agent Reverse Proxy）让这一切变得简单：

```
☁️ 咖啡厅、机场、酒店...      →    🏢 公司内网的 Claude Code
📱 手机、平板、笔记本...      →    🏠 家中的 AI 开发环境
🌍 全球任何网络位置...        →    🔒 NAT/防火墙后的智能体
```

**无需配置路由器、无需 VPN、无需公网 IP** — 只需一个中转服务器。

---

## 💡 典型场景

### 场景 1: 移动办公

```
在咖啡厅用笔记本/手机              公司内网服务器
     💻 📱                            🖥️
     │                                 │
     │ 打开 console.agentx.plus        │  Claude Code
     │ 填入服务器地址 + Token           │  (内网 192.168.1.100)
     │                                 │
     └────── Internet ─────►  ☁️  ─────►
             公网访问         ARP服务器
```

**效果**：在星巴克打开手机浏览器，访问 Web UI，就能调用公司内网的 Claude Code，就像在办公室一样。

### 场景 2: 远程协作

```
团队成员 A (北京)  ─┐
团队成员 B (上海)  ─┤
客户 (纽约)        ─┤──►  ☁️ ARP  ──►  🏢 您的 AI 智能体
                   │    公网入口      内网(无公网IP)
合作伙伴 (伦敦)    ─┘
```

**效果**：所有人通过统一 URL 访问，无需每个人配置 VPN。

### 场景 3: 多设备无缝切换

```
早上: 台式机(家中) ──►  ☁️ ARP  ──►  🏢 内网 Claude Code
午休: 手机(公司)   ──►     │
晚上: 平板(咖啡厅) ──►     │
              同一个会话，随时续接
```

**效果**：工作连续性，会话状态自动保持。

---

## 🚀 5 分钟快速开始

### 步骤 1: 安装服务器端（arps）

在有公网 IP 的服务器上运行（云主机、VPS、家用服务器 + DDNS...）

```bash
# 一键安装并启动
curl -fsSL https://s3.agentx.plus/install.sh | bash
```

<details>
<summary>📦 或者从源码编译（点击展开）</summary>

```bash
git clone <repo>
cd agentx_proxy
cargo build --release

./target/release/arps \
  --control-port 17001 \
  --proxy-port 17002 \
  --public-port 17003
```
</details>

安装完成后，服务器将监听：
- **17001** - 控制端口（客户端注册）
- **17002** - 代理端口（隧道建立）
- **17003** - 公网端口（对外访问入口）

### 步骤 2: 安装客户端（arpc）

在内网机器上运行（这台机器运行您的 AI 智能体）

#### Linux / macOS

```bash
# 一键安装
curl -fsSL https://s3.agentx.plus/install-client.sh | bash

# 启动客户端（命令模式 - 代理 Claude Code 等智能体）
arpc --server-addr <您的公网服务器IP> --command-mode

# 输出示例：
# ✅ Starting agentc with client_id: 6D13F98FCCCD5F089AD96C3C7C13A81C
# 🔗 Successfully registered with the server.
# 🌐 Public URL: http://<服务器IP>:17003?token=a1b2c3d4
```

#### Windows

```powershell
# 在 PowerShell 中运行
irm https://s3.agentx.plus/install-client.ps1 | iex

# 启动客户端
arpc --server-addr <您的公网服务器IP> --command-mode
```

<details>
<summary>📦 或者从源码运行（点击展开）</summary>

```bash
git clone <repo>
cd agentx_proxy

cargo run -p arpc -- \
  --server-addr <您的公网服务器IP> \
  --command-mode
```
</details>

### 步骤 3: 从任何地方访问

现在您可以在**手机、咖啡厅、机场、酒店...任何有网络的地方**访问内网智能体！

#### 🖥️ 方式 1：Web UI（推荐）

打开浏览器访问 Web 管理界面，无需任何技术背景：
```
# 浏览器直接访问（默认连的是我的测试服务器）
https://console.agentx.plus/?token=（客户端启动时输出的 client_id）
```
或者

```
# 浏览器访问
https://console.agentx.plus
```

在界面中填入：
- **服务器地址**：`<您的公网服务器IP>:17003`（或您的域名）
- **Token**：`a1b2c3d4`（客户端启动时输出的 client_id）

点击连接即可：
- 📂 浏览 Claude 项目和会话历史
- 💬 创建新的智能体执行会话
- 📊 查看会话状态和实时输出
- 🔄 管理正在运行的任务

**适合**：非技术用户、移动设备、快速演示

#### 🔧 方式 2：HTTP API（开发者）

通过 API 集成到您的应用中：

```bash
# 在任何设备上：
curl "http://<服务器IP>:17003/api/claude/projects?token=a1b2c3d4"

# 创建新会话
curl -X POST "http://<服务器IP>:17003/api/sessions?token=a1b2c3d4" \
  -H "Content-Type: application/json" \
  -d '{"executor":"Claude","prompt":"分析这个项目"}'
```

**适合**：开发者集成、自动化脚本、CI/CD 流程

✨ **就这么简单！**

---

## 🔐 为什么不直接暴露内网服务？

| 方案 | 复杂度 | 安全性 | 灵活性 |
|------|--------|--------|--------|
| **端口转发** | ❌ 需要路由器配置<br>❌ 每次网络变化都要重配 | ⚠️ 直接暴露内网 | ❌ 固定IP+端口 |
| **VPN** | ❌ 每个访问者都要配置<br>❌ 移动设备配置麻烦 | ✅ 加密通道 | ⚠️ 需要客户端 |
| **ARP** | ✅ 一次部署，到处访问 | ✅ 仅暴露代理隧道<br>✅ Token 隔离 | ✅ 统一HTTP入口<br>✅ 支持所有设备 |

---

## 🎨 核心特性

### 🌐 真正的"随时随地"访问

- ✅ **跨网络**：咖啡厅 WiFi、4G/5G、酒店网络...任何能上网的地方
- ✅ **跨设备**：手机、平板、笔记本、台式机，浏览器即可访问
- ✅ **跨平台**：Windows、macOS、Linux、iOS、Android 通吃
- ✅ **零学习成本**：提供 Web UI（https://console.agentx.plus），填入服务器地址和 Token 即可使用

### ⚡ 低延迟设计

- **连接池预热**：代理隧道预先建立，平均响应 < 50ms
- **零拷贝转发**：Rust 异步 I/O，减少内存拷贝
- **TCP 优化**：`TCP_NODELAY` + 大缓冲区

### 🛡️ 安全隔离

- 内网服务不直接暴露公网
- 每个客户端独立 `client_id` Token
- 支持 HTTPS/WSS（通过 nginx 前置）

### 🔌 多智能体支持

同时代理多个 AI 智能体：

```bash
# 机器 A：Claude Code
arpc --client-id claude-agent --command-mode

# 机器 B：本地 LLM API
arpc --client-id llm-api --local-port 8000

# 访问：
# http://server:17003/api/sessions?token=claude-agent
# http://server:17003/anything?token=llm-api
```

---

## 📡 工作原理

```
┌─────────────────────────────────────────────────────────┐
│  Step 1: 客户端注册（启动时自动完成）                      │
└─────────────────────────────────────────────────────────┘

内网客户端                      公网服务器
  arpc                          arps
    │                               │
    │  ① Register {client_id}       │
    ├──────────────────────────────►│
    │                               │
    │  ② RegisterResult {success}   │
    │◄──────────────────────────────┤
    │                               │
    │  ③ 建立代理连接池(预热)         │
    ├══════════════════════════════►│
    ├══════════════════════════════►│


┌─────────────────────────────────────────────────────────┐
│  Step 2: 外部访问（从任何地方）                            │
└─────────────────────────────────────────────────────────┘

    手机/笔记本                  公网服务器              内网客户端
  (咖啡厅/机场)                   arps                 arpc
       │                            │                      │
       │  GET /api/sessions         │                      │
       │  ?token=client_id          │                      │
       ├───────────────────────────►│                      │
       │                            │                      │
       │                            │  ④ 从连接池取出隧道   │
       │                            ├═════════════════════►│
       │                            │                      │
       │                            │  ⑤ 请求转发到AI智能体  │
       │                            │                      ├──► Claude Code
       │                            │                      │
       │                            │  ⑥ 响应返回          │
       │                            │◄═════════════════════┤
       │  ⑦ HTTP Response           │                      │
       │◄───────────────────────────┤                      │
```

**关键点**：
- 内网客户端**主动连接**公网服务器（不需要公网 IP）
- 连接池机制：预先建立多条隧道，外部请求到达时立即复用
- Token 路由：通过 `?token=<client_id>` 将请求路由到对应内网客户端

---

## 📚 支持的 AI 智能体

### 开箱即用

- ✅ **Claude Code** - Anthropic 官方 CLI
- ✅ **Codex** - OpenAI Codex CLI
- ✅ **Gemini** - Google Gemini CLI（待完善）

### API 能力

当客户端运行在 `--command-mode`，提供 HTTP API：

#### 会话管理

```bash
# 创建执行会话
POST /api/sessions?token=<client_id>
{
  "executor": "Claude",
  "model": "claude-sonnet-4.5",
  "prompt": "帮我重构这个函数",
  "projectPath": "/home/user/myproject"
}

# 查询会话状态
GET /api/sessions/{session_id}?token=<client_id>

# 取消/删除会话
DELETE /api/sessions/{session_id}?token=<client_id>
```

#### Claude 专属功能

```bash
# 列出本地 Claude 项目
GET /api/claude/projects?token=<client_id>

# 查询历史会话
GET /api/claude/sessions?token=<client_id>

# 加载会话消息
GET /api/claude/sessions/{session_id}?token=<client_id>
```

### 纯转发模式

也可以代理任何 TCP 服务（Web 应用、API、数据库...）：

```bash
cargo run -p arpc -- \
  --server-addr <公网IP> \
  --local-addr 127.0.0.1 \
  --local-port 3000  # 本地服务端口
```

访问：`http://<公网IP>:17003?token=<client_id>` → 自动转发到内网 `localhost:3000`

---

## 🏗️ 生产部署

### 服务器端（arps）

使用 systemd 管理：

```bash
# 1. 构建
cargo build --release

# 2. 安装服务
sudo scripts/install_server.sh

# 3. 配置端口（可选）
sudo systemctl edit arps
# 添加环境变量：
[Service]
Environment="PUBLIC_PORT=80"  # 使用标准 HTTP 端口

# 4. 启动
sudo systemctl start arps
sudo systemctl enable arps  # 开机自启

# 5. 查看日志
journalctl -u arps -f
```

### 客户端（arpc）

#### Linux/macOS

```bash
# 添加到 crontab
crontab -e

# 添加行：
@reboot cd /path/to/agentx_proxy && cargo run -p arpc -- \
  --server-addr <IP> --command-mode >> /tmp/arpc.log 2>&1
```

#### Windows

创建 `startup.bat` 并加到启动项：

```batch
cd C:\path\to\agentx_proxy
cargo run -p arpc -- --server-addr <IP> --command-mode
```

### 配置 HTTPS（推荐）

在 arps 前部署 nginx：

```nginx
server {
    listen 443 ssl;
    server_name your-domain.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://127.0.0.1:17003;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

现在可以通过 HTTPS 访问：
```
https://your-domain.com/api/sessions?token=<client_id>
```

---

## 🎮 实战示例

### 示例 1: 移动端访问内网 Claude Code

**场景**：在咖啡厅用手机访问公司内网的 Claude Code

#### 使用 Web UI（最简单）

```bash
# 1. 公司内网机器启动客户端
arpc --server-addr proxy.example.com --command-mode
# 输出 token: abc123

# 2. 手机浏览器打开
https://console.agentx.plus

# 3. 填写连接信息
服务器地址: proxy.example.com:17003
Token: abc123

# 4. 点击"连接"后，即可：
# - 查看 Claude 项目列表
# - 创建新会话："帮我分析公司项目架构"
# - 实时查看执行输出
```

#### 使用 API（开发者集成）

```bash
# 1. 列出项目
curl "https://proxy.example.com:17003/api/claude/projects?token=abc123"

# 2. 创建会话
curl -X POST "https://proxy.example.com:17003/api/sessions?token=abc123" \
  -H "Content-Type: application/json" \
  -d '{
    "executor": "Claude",
    "prompt": "分析当前目录的代码结构"
  }'

# 3. 查看输出
curl "https://proxy.example.com:17003/api/sessions/{session_id}?token=abc123"
```

### 示例 2: 多团队共享内网智能体

```bash
# 公司服务器 A：代理团队的 Claude Code
arpc --client-id team-a --command-mode

# 公司服务器 B：代理团队的自定义 LLM
arpc --client-id team-b --local-port 8000

# 团队 A 成员访问：
https://proxy.company.com/api/sessions?token=team-a

# 团队 B 成员访问：
https://proxy.company.com/chat?token=team-b
```

### 示例 3: 家庭网络的 AI 助手

```bash
# 家中 NAS/树莓派 运行客户端
arpc --server-addr home-proxy.ddns.net --command-mode

# 外出时用手机访问：
# - 在地铁上查看 Claude 项目进度
# - 在咖啡厅创建新的代码分析任务
# - 在酒店续接早上未完成的会话
```

---

## 🔧 高级配置

### 自定义 client_id

```bash
# 固定 ID（方便记忆和配置）
arpc --client-id my-workspace --server-addr <IP> --command-mode

# 访问 URL
http://<IP>:17003/api/sessions?token=my-workspace
```

### 调整连接池大小

```bash
# 服务器端：每个客户端预热 5 个连接
arps --pool-size 5

# 高并发场景建议 5-10
# 低频访问场景建议 1-2
```

### 指定 Claude 命令路径

```bash
arpc --command-path "/opt/claude/bin/claude" --command-mode
```

### 启用调试日志

```bash
RUST_LOG=debug cargo run -p arpc -- <参数>
```

---

## 📊 性能数据

测试环境：公网服务器（阿里云 ECS）+ 内网客户端（家庭宽带）

| 指标 | 连接池命中 | 连接池未命中 |
|------|-----------|-------------|
| **响应延迟** | 45ms | 180ms |
| **吞吐量** | ~500 req/s | ~150 req/s |
| **内存占用** | arps: 20MB<br>arpc: 15MB | - |

**结论**：连接池机制大幅降低延迟，适合交互式场景。

---

## ❓ 常见问题

### Q: 手机 4G 网络可以访问吗？

A: ✅ 可以！只要能访问公网服务器即可，任何网络都支持（WiFi、4G、5G...）。

### Q: 多人同时访问会冲突吗？

A: ❌ 不会。每个请求独立处理，支持并发（受限于 `--pool-size` 配置）。

### Q: 内网客户端断线怎么办？

A: 客户端会自动重连。服务器在连接断开后保留客户端信息 5 分钟。

### Q: 如何保证安全性？

A:
1. 使用 HTTPS（nginx + Let's Encrypt）
2. 在 arps 前配置防火墙/安全组
3. 为敏感 API 添加额外认证（JWT、API Key）
4. 定期轮换 client_id

### Q: 支持 WebSocket 吗？

A: ✅ 支持！public-port 可以升级到 WebSocket，实现会话流式输出。

### Q: 和 frp、ngrok 有什么区别？

A:
- **frp/ngrok**：通用端口转发工具
- **ARP**：专为 AI 智能体优化，内置会话管理、Claude 集成、连接池等

### Q: 客户端日志在哪里？

A: 日志文件保存在：

| 操作系统    | 日志目录路径                                   |
|---------|------------------------------------------|
| Windows | C:\Users\<用户名>\AppData\Local\arpc\logs\  |
| Linux   | ~/.local/share/arpc/logs/                |
| macOS   | ~/Library/Application Support/arpc/logs/ |

日志文件名格式为 arpc.log.YYYY-MM-DD，每天自动滚动创建新文件。

如果无法访问系统目录，会回退到当前工作目录下的 ./arpc/logs/。


---

## 🛣️ 路线图

- [ ] Web 管理面板（查看在线客户端、流量统计）
- [ ] 内置认证系统（不依赖 Token）
- [ ] 端到端加密（E2EE）
- [ ] Docker 镜像和 Kubernetes Helm Chart
- [ ] 客户端自动更新机制

---

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

贡献前请：
- 运行 `cargo fmt --all` 格式化
- 运行 `cargo clippy` 通过检查
- 为新功能编写测试

---

## 📄 许可证

MIT License

---

<div align="center">

**ARP - 让内网 AI 智能体，随时随地触手可及** 🌐

让距离不再是问题 | 让网络不再是障碍 | 让 AI 无处不在

</div>
