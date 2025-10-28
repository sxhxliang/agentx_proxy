# AgentX Proxy 测试与演示脚本

本文档介绍 AgentX Proxy 项目中提供的测试和演示脚本，帮助您快速体验和测试系统的各项功能。

## 📁 脚本列表

### 1. `quick_demo.sh` - 快速演示
最简单的方式体验 AgentX Proxy 的基本功能。

**功能**:
- 启动测试 HTTP 服务 (端口 8080)
- 启动 AgentX 服务器
- 启动 AgentX 客户端
- 自动测试连接
- 显示访问地址

**用法**:
```bash
# 赋予执行权限
chmod +x scripts/quick_demo.sh

# 运行快速演示
./scripts/quick_demo.sh
```

**输出示例**:
```
🎉 演示成功！

访问信息:
  本地服务地址:  http://localhost:8080
  代理访问地址:  http://localhost:17003/?token=demo-client-1234567890
```

### 2. `setup_demo.sh` - 一键演示环境
提供多种演示场景的交互式设置脚本。

**功能**:
- 5 种预设演示场景
- 交互式场景选择
- 自动服务管理
- 详细的访问信息

**用法**:
```bash
# 交互式选择
./scripts/setup_demo.sh

# 直接运行特定场景
./scripts/setup_demo.sh -s 1  # Web 开发环境
./scripts/setup_demo.sh -s 2  # API 服务
./scripts/setup_demo.sh -s 3  # 数据库访问
./scripts/setup_demo.sh -s 4  # Claude 命令执行
./scripts/setup_demo.sh -s 5  # 完整演示

# 使用自定义配置
POOL_SIZE=10 ./scripts/setup_demo.sh -s 1
```

**场景说明**:

| 场景 | 描述 | 本地端口 | 特点 |
|------|------|----------|------|
| 1 - Web 开发 | 将本地 Web 服务器暴露到公网 | 3000 | 适用于前端/全栈开发 |
| 2 - API 服务 | 暴露本地 REST API | 8000 | 适用于 API 测试和分享 |
| 3 - 数据库访问 | 暴露本地数据库服务 | 5432 | 适用于远程数据库管理 |
| 4 - Claude 命令执行 | 启用 HTTP API 和命令模式 | N/A | 支持会话管理和命令执行 |
| 5 - 完整演示 | 运行所有场景 | 全部 | 全面展示系统能力 |

**配置选项**:
```bash
--pool-size SIZE        # 连接池大小 (默认: 3)
--server-host HOST      # 服务器地址 (默认: 0.0.0.0)
--ports CTRL:PROXY:PUB  # 端口配置 (默认: 17001:17002:17003)
```

### 3. `test_scenarios.sh` - 完整测试场景
全面的功能测试脚本，验证所有使用场景。

**功能**:
- 场景 1: TCP 代理模式测试
- 场景 2: 命令模式 API 测试
- 场景 3: 多客户端并发测试
- 场景 4: 数据库模拟测试
- 场景 5: 连接池压力测试

**用法**:
```bash
# 运行所有测试
./scripts/test_scenarios.sh

# 运行特定测试
./scripts/test_scenarios.sh <<EOF
2
EOF

# 或直接选择
./scripts/test_scenarios.sh
# 然后在提示符选择 1-6
```

**测试场景详解**:

#### 场景 1: TCP 代理模式
```bash
# 测试内容:
# 1. 启动测试 HTTP 服务 (端口 8000)
# 2. 启动 AgentX 客户端
# 3. 验证通过 token 访问
# 4. 测试代理连接
```
**预期结果**:
- ✅ 本地服务可以正常访问
- ✅ 代理地址可以正常访问
- ✅ 返回正确的 HTTP 内容

#### 场景 2: 命令模式
```bash
# 测试内容:
# 1. 启动命令模式客户端
# 2. 测试 POST /api/sessions API
# 3. 测试 GET /api/sessions/{id} API
# 4. 验证会话管理
```
**预期结果**:
- ✅ API 返回正确的 JSON
- ✅ 会话 ID 生成成功
- ✅ 会话状态查询正常

#### 场景 3: 多客户端
```bash
# 测试内容:
# 1. 同时启动 3 个客户端实例
# 2. 每个客户端连接不同端口
# 3. 并发测试所有客户端
```
**预期结果**:
- ✅ 所有客户端成功连接
- ✅ 每个客户端独立工作
- ✅ 访问地址各不相同

#### 场景 4: 数据库模拟
```bash
# 测试内容:
# 1. 启动模拟数据库 (netcat)
# 2. 暴露数据库端口 (5432)
# 3. 验证 TCP 代理
```
**预期结果**:
- ✅ 数据库代理测试成功
- ✅ 显示连接命令示例

#### 场景 5: 压力测试
```bash
# 测试内容:
# 1. 启动 5 个客户端实例
# 2. 并发发送 10 个请求
# 3. 测试连接池性能
```
**预期结果**:
- ✅ 所有请求成功处理
- ✅ 无错误或超时
- ✅ 并发性能符合预期

### 4. `benchmark.sh` - 性能测试
系统性能基准测试脚本。

**功能**:
- 并发性能测试
- 连接池优化测试
- 长期稳定性测试
- 详细性能报告

**用法**:
```bash
# 基本性能测试
./scripts/benchmark.sh

# 自定义配置
./scripts/benchmark.sh -p 10 -d 60 -c 20

# 稳定性测试
./scripts/benchmark.sh --stability -d 120

# 使用环境变量
POOL_SIZE=10 TEST_DURATION=60 CONCURRENT_USERS=20 ./scripts/benchmark.sh
```

**参数说明**:
```
-p, --pool-size SIZE    # 连接池大小 (默认: 5)
-d, --duration SEC      # 测试持续时间 (默认: 30 秒)
-c, --concurrent NUM    # 并发用户数 (默认: 10)
-s, --stability         # 运行稳定性测试
```

**输出示例**:
```
========================================
   AgentX Proxy 性能测试
========================================

配置:
  连接池大小: 10
  测试持续时间: 60s
  并发用户数: 20

[INFO] 启动 AgentX 服务器 (连接池大小: 10)...
[SUCCESS] 服务器已启动 (PID: 12345)

[INFO] 测试结果:
  总请求数: 60
  成功: 60
  失败: 0
  耗时: 3.2s
  平均 RPS: 18.75

[SUCCESS] 性能测试完成
```

**测试指标**:
- **RPS** (Requests Per Second): 每秒处理请求数
- **延迟**: 请求响应时间
- **成功率**: 成功请求的百分比
- **连接池优化**: 预热连接池带来的性能提升

## 🔧 通用脚本功能

### 依赖检查
所有脚本都会自动检查:
- ✅ Rust/Cargo 是否安装
- ✅ 必要的系统工具 (curl, lsof, nc, python3)
- ✅ 项目是否已编译
- ✅ 端口是否可用

### 进程管理
脚本会自动:
- 🚀 启动必要的后台进程
- 📊 监控进程状态
- 🛑 优雅停止所有进程
- 🧹 清理临时文件

### 日志记录
- 所有日志保存到 `/tmp/` 目录
- 每个进程有独立的日志文件
- 脚本运行日志带时间戳

**日志文件位置**:
```
/tmp/agents.log              # 服务器日志
/tmp/agentc_<client_id>.log  # 客户端日志
/tmp/test_server_<port>.log  # 测试服务日志
/tmp/benchmark_*.json        # 性能测试结果
```

### 错误处理
- 🔍 自动检测服务启动失败
- 📝 提供详细的错误日志
- 🔄 失败时自动清理资源
- ⚠️ 友好的错误提示信息

## 📊 测试结果解读

### 成功指标
```
✅ 所有测试通过
✅ 连接成功率 > 95%
✅ 响应时间 < 100ms
✅ 无内存泄漏或崩溃
```

### 警告指标
```
⚠️  连接成功率 80-95%
⚠️  响应时间 100-500ms
⚠️  偶尔超时或重试
```

### 失败指标
```
❌ 连接成功率 < 80%
❌ 响应时间 > 500ms
❌ 频繁超时或错误
❌ 进程崩溃
```

## 🚀 性能调优建议

### 连接池大小
```bash
# 低并发 (1-10 用户)
POOL_SIZE=2

# 中等并发 (10-50 用户)
POOL_SIZE=5

# 高并发 (50+ 用户)
POOL_SIZE=10
```

### TCP 缓冲区
编辑 `agents/src/main.rs`:
```rust
// 增大缓冲区 (默认 256KB)
setsockopt(socket.as_socket(), socket2::Socket::SO_RCVBUF, &(512 * 1024))?;
setsockopt(socket.as_socket(), socket2::Socket::SO_SNDBUF, &(512 * 1024))?;
```

### 系统限制
```bash
# 增加文件描述符限制
ulimit -n 65536

# 优化网络内核参数
echo 'net.core.rmem_max = 134217728' >> /etc/sysctl.conf
echo 'net.core.wmem_max = 134217728' >> /etc/sysctl.conf
sysctl -p
```

## 🐛 故障排除

### 常见问题

#### 1. 端口被占用
```bash
# 查看端口占用
lsof -i :17001

# 杀死占用进程
kill -9 <PID>

# 或使用脚本自动处理
./scripts/setup_demo.sh  # 脚本会自动清理
```

#### 2. Cargo 命令未找到
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 验证安装
cargo --version
```

#### 3. 客户端连接失败
```bash
# 启用调试日志
RUST_LOG=debug ./scripts/quick_demo.sh

# 查看日志
tail -f /tmp/agentc_*.log
tail -f /tmp/agents.log
```

#### 4. 测试服务启动失败
```bash
# 检查 Python 版本
python3 --version

# 手动启动测试服务
python3 -m http.server 8080

# 检查端口是否被占用
netstat -tuln | grep 8080
```

#### 5. 权限问题
```bash
# 添加执行权限
chmod +x scripts/*.sh

# 如果仍然失败，尝试使用 bash
bash scripts/quick_demo.sh
```

### 日志调试
```bash
# 查看所有日志
tail -f /tmp/agents.log
tail -f /tmp/agentc_*.log

# 查看最近的错误
grep -i error /tmp/agents.log
grep -i error /tmp/agentc_*.log

# 性能问题诊断
# 检查 CPU 和内存使用
top -p $(pgrep -f "cargo run")

# 检查网络连接
netstat -an | grep 17001
netstat -an | grep 17002
netstat -an | grep 17003
```

## 📝 自定义测试

### 创建自定义测试服务
```python
# 创建自定义测试服务
cat > /tmp/my_service.py <<'EOF'
from http.server import HTTPServer, BaseHTTPRequestHandler

class MyHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header('Content-type', 'text/html')
        self.end_headers()
        self.wfile.write(b'<h1>My Custom Service</h1>')

if __name__ == '__main__':
    server = HTTPServer(('127.0.0.1', 9000), MyHandler)
    server.serve_forever()
EOF

# 启动服务
python3 /tmp/my_service.py &

# 使用 AgentX 暴露
cargo run -p agentc -- \
    --client-id my-service \
    --local-port 9000 \
    # ... 其他参数
```

### 编写自定义测试脚本
```bash
#!/bin/bash
# my_custom_test.sh

# 启动服务器
cargo run -p agents -- --pool-size 5 &

# 等待启动
sleep 3

# 启动测试服务
python3 -m http.server 9000 &

# 启动客户端
cargo run -p agentc -- \
    --client-id custom-test \
    --local-port 9000 &

# 等待连接
sleep 3

# 运行测试
curl -v "http://localhost:17003/?token=custom-test"

# 清理
pkill -f "cargo run"
pkill -f "python3 -m http.server"
```

## 🎯 使用场景快速参考

| 需求 | 推荐脚本 | 场景 |
|------|----------|------|
| 快速体验 | `quick_demo.sh` | 5 分钟了解基本功能 |
| 功能验证 | `test_scenarios.sh` | 验证所有功能 |
| 性能评估 | `benchmark.sh` | 性能基准测试 |
| 产品演示 | `setup_demo.sh` | 演示给客户/团队 |
| 压力测试 | `benchmark.sh --stability` | 长期稳定性测试 |

## 📚 相关文档

- [项目 README](../README.md) - 项目总体介绍
- [架构文档](../CLAUDE.md) - 详细的架构说明
- [API 文档](../README.md#api-参考) - API 接口说明

## 🤝 贡献

如果您创建了新的测试脚本或有改进建议，欢迎提交 Pull Request！

创建新脚本时，请遵循:
1. 添加 shebang: `#!/bin/bash`
2. 使用颜色输出提高可读性
3. 添加详细的帮助信息
4. 实现优雅的错误处理
5. 提供清晰的使用示例

---

**提示**: 所有脚本都支持 Ctrl+C 优雅退出，脚本会自动清理所有后台进程。
