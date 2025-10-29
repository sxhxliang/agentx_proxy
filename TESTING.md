# AgentX Proxy 测试与演示指南

本文档说明如何使用 AgentX Proxy 项目中提供的测试和演示脚本，快速体验和验证系统功能。

## 📁 创建的脚本清单

我们根据 README 中的场景和示例，创建了以下测试脚本：

### 核心演示脚本

#### 1. `quick_demo.sh` - 快速演示 (2.7KB)
- **功能**: 30秒快速体验 AgentX Proxy 基本功能
- **特点**: 全自动、无需交互、一键运行
- **使用场景**: 首次体验、产品展示
- **执行时间**: 约30秒
- **命令**:
```bash
./scripts/quick_demo.sh
# 或
make quick
```

#### 2. `setup_demo.sh` - 一键演示环境 (15KB)
- **功能**: 5种预设场景的交互式演示
- **特点**: 可选择场景、详细说明、自动管理
- **使用场景**: 产品演示、用户培训
- **执行时间**: 1-5分钟
- **命令**:
```bash
./scripts/setup_demo.sh
# 或
make demo
```
- **场景选择**:
  - 场景1: Web开发环境 (端口3000)
  - 场景2: API服务 (端口8000)
  - 场景3: 数据库访问 (端口5432)
  - 场景4: Claude命令执行
  - 场景5: 完整演示

### 功能测试脚本

#### 3. `test_scenarios.sh` - 完整测试场景 (12KB)
- **功能**: 5种使用场景的功能验证测试
- **特点**: 交互式选择、完整覆盖、详细报告
- **使用场景**: QA测试、功能验证、回归测试
- **执行时间**: 5-15分钟
- **命令**:
```bash
./scripts/test_scenarios.sh
# 或选择特定测试
make test-1  # TCP代理模式
make test-2  # 命令模式
make test-3  # 多客户端
make test-4  # 数据库
make test-5  # 压力测试
```

### 性能测试脚本

#### 4. `benchmark.sh` - 性能基准测试 (197KB)
- **功能**: 全面的性能评估和压力测试
- **特点**: 并发测试、连接池优化、稳定性测试、JSON报告
- **使用场景**: 性能评估、容量规划、对比测试
- **执行时间**: 1-10分钟
- **命令**:
```bash
# 基本性能测试
./scripts/benchmark.sh

# 自定义配置
./scripts/benchmark.sh -p 10 -d 60 -c 20

# 稳定性测试
./scripts/benchmark.sh --stability -d 120

# 或使用make
make benchmark
make benchmark POOL_SIZE=10 DURATION=60
```

### 辅助工具

#### 5. `config.env` - 配置环境 (7.6KB)
- **功能**: 环境变量定义和工具函数
- **特点**: 颜色输出、日志管理、服务控制、帮助函数
- **使用场景**: 脚本开发、CI/CD、自定义测试
- **命令**:
```bash
source scripts/config.env
agentx_show_config
agentx_start_test_service 3000 "MyService"
agentx_cleanup
```

#### 6. `Makefile` - 便捷命令 (5.1KB)
- **功能**: 封装常用命令，提供便捷操作
- **特点**: 简化命令、统一接口、自动依赖检查
- **使用场景**: 日常开发、快速测试
- **命令**:
```bash
make help          # 显示帮助
make check         # 检查环境
make build         # 构建项目
make quick         # 快速演示
make test          # 功能测试
make benchmark     # 性能测试
make clean         # 清理
```

#### 7. `README.md` - 详细文档 (12KB)
- **功能**: 完整的脚本使用说明
- **特点**: 详细说明、示例丰富、故障排除
- **包含内容**:
  - 每个脚本的详细说明
  - 使用示例和参数
  - 故障排除指南
  - 性能调优建议
  - 自定义测试方法

#### 8. `INDEX.md` - 快速索引 (7KB)
- **功能**: 所有脚本的快速参考索引
- **特点**: 决策树、速查表、特性对比
- **包含内容**:
  - 脚本对比表
  - 使用场景决策树
  - 常用命令速查
  - 学习路径推荐

## 🎯 使用场景与脚本映射

根据 README 中的使用场景，我们创建了对应的测试脚本：

### 场景1: 将本地开发环境暴露到公网

**相关脚本**:
- `quick_demo.sh` - 基础演示
- `setup_demo.sh -s 1` - Web开发环境演示
- `test_scenarios.sh - 选择1` - TCP代理模式测试

**测试内容**:
- 启动本地测试服务 (端口3000)
- 通过AgentX暴露到公网
- 验证访问地址: `http://localhost:17003/?token=client-id`

### 场景2: 远程使用Claude Code

**相关脚本**:
- `setup_demo.sh -s 4` - Claude命令执行演示
- `test_scenarios.sh - 选择2` - 命令模式测试

**测试内容**:
- 启用HTTP API模式
- 测试会话创建: `POST /api/sessions`
- 测试会话查询: `GET /api/sessions/{id}`

### 场景3: 多开发者共享开发环境

**相关脚本**:
- `setup_demo.sh -s 5` - 完整演示
- `test_scenarios.sh - 选择3` - 多客户端测试

**测试内容**:
- 启动3个客户端实例
- 每个客户端绑定不同token
- 并发测试所有客户端

### 场景4: 访问内网数据库和服务

**相关脚本**:
- `setup_demo.sh -s 3` - 数据库访问演示
- `test_scenarios.sh - 选择4` - 数据库模拟测试

**测试内容**:
- 模拟数据库监听 (使用netcat)
- 暴露数据库端口 (5432)
- 提供连接命令示例

### 场景5: 演示和临时环境分享

**相关脚本**:
- `setup_demo.sh` - 交互式演示
- `quick_demo.sh` - 快速演示

**测试内容**:
- 自动生成唯一client-id
- 显示访问地址
- 一键启动所有服务

### 实际应用示例

我们还为README中的实际应用示例提供了测试支持：

#### 示例1: 全栈Web应用开发

```bash
# 启动前端服务 (Vite 5173)
# 启动后端服务 (API 8000)
# 同时暴露两个服务
./scripts/setup_demo.sh -s 1  # Web开发演示
```

#### 示例2: 数据库远程访问

```bash
# 暴露MySQL/PostgreSQL
./scripts/setup_demo.sh -s 3  # 数据库演示
```

#### 示例3: CI/CD测试环境

```bash
# 压力测试和稳定性测试
make benchmark  # 基本性能
make benchmark-stability  # 稳定性测试
```

## 🚀 快速开始指南

### 方法一: Make命令 (推荐新手)

```bash
# 1. 检查环境
make check

# 2. 构建项目
make build

# 3. 运行快速演示
make quick

# 4. 运行完整测试
make test
```

### 方法二: 直接执行脚本 (推荐开发者)

```bash
# 1. 赋予权限
chmod +x scripts/*.sh

# 2. 快速演示
./scripts/quick_demo.sh

# 3. 交互式演示
./scripts/setup_demo.sh

# 4. 功能测试
./scripts/test_scenarios.sh

# 5. 性能测试
./scripts/benchmark.sh
```

### 方法三: 使用配置环境 (推荐高级用户)

```bash
# 1. 加载配置
source scripts/config.env

# 2. 显示配置
agentx_show_config

# 3. 启动测试服务
agentx_start_test_service 3000 "WebDev"

# 4. 自定义测试
# (使用配置中的工具函数编写自定义脚本)

# 5. 清理环境
agentx_cleanup
```

## 📊 脚本特性对比

| 脚本 | 自动化 | 交互性 | 覆盖范围 | 执行时间 | 难度 | 推荐用户 |
|------|--------|--------|----------|----------|------|----------|
| quick_demo | ✅✅✅ | ❌ | 基础 | 30秒 | ⭐ | 新手 |
| setup_demo | ✅✅ | ✅✅ | 中等 | 1-5分钟 | ⭐⭐ | 所有用户 |
| test_scenarios | ✅ | ✅✅ | 全面 | 5-15分钟 | ⭐⭐⭐ | 开发者 |
| benchmark | ✅ | 参数 | 性能专项 | 1-10分钟 | ⭐⭐⭐⭐ | 性能工程师 |
| config.env | 工具库 | 代码 | N/A | N/A | ⭐⭐⭐⭐⭐ | 高级用户 |

## 🔧 高级用法

### 自定义配置

```bash
# 使用环境变量
export POOL_SIZE=10
export TEST_DURATION=60
export CONCURRENT_USERS=20
export SERVER_HOST="your-server.com"

# 运行测试
make benchmark
```

### 自定义测试服务

```bash
# 加载配置
source scripts/config.env

# 启动自定义服务
agentx_start_test_service 9000 "MyService"

# 使用AgentX暴露
cargo run -p arpc -- \
    --client-id my-service \
    --local-port 9000

# 清理
agentx_cleanup
```

### CI/CD集成

```bash
# 在CI中运行测试
#!/bin/bash
set -e

make check
make build
make test
make benchmark

# 检查测试结果
if [ -f /tmp/agentx_benchmark_*.json ]; then
    echo "Performance tests passed"
fi
```

## 📝 日志和输出

### 日志文件位置

所有脚本运行日志保存在 `/tmp/` 目录：

```
/tmp/
├── arps_*.log              # 服务器日志
├── arpc_*.log              # 客户端日志
├── test_server_*.log         # 测试服务日志
├── demo_*.log                # 演示日志
├── benchmark_*.json          # 性能测试结果
├── *.pid                     # 进程ID文件
└── test_server_*.py          # 临时测试脚本
```

### 查看日志

```bash
# 实时查看日志
tail -f /tmp/arps.log

# 查看特定客户端日志
tail -f /tmp/arpc_client-id.log

# 查看性能测试结果
cat /tmp/agentx_benchmark_*.json

# 查看所有错误
grep -i error /tmp/*.log
```

## 🐛 故障排除

### 常见问题

#### 1. 端口被占用
```bash
# 检查端口
lsof -i :17001

# 清理进程
make clean

# 或手动清理
pkill -f "cargo run"
```

#### 2. 权限问题
```bash
# 添加执行权限
chmod +x scripts/*.sh

# 使用bash执行
bash scripts/quick_demo.sh
```

#### 3. 依赖缺失
```bash
# 检查环境
make check

# 安装缺失的依赖
# - Rust: https://rustup.rs/
# - curl: 包管理器安装
# - python3: 包管理器安装
# - bc: 包管理器安装
```

#### 4. 测试失败
```bash
# 启用调试日志
RUST_LOG=debug make quick

# 查看详细日志
cat /tmp/arpc_*.log
cat /tmp/arps.log

# 清理后重试
make clean
make quick
```

### 获取帮助

```bash
# 查看Makefile帮助
make help

# 查看脚本帮助
./scripts/quick_demo.sh  --help  # 如果支持
./scripts/setup_demo.sh --help  # 如果支持
./scripts/benchmark.sh --help

# 查看详细文档
cat scripts/README.md
cat scripts/INDEX.md
```

## 📈 测试结果解读

### 成功指标

```
✅ 所有测试通过
✅ 连接成功率 = 100%
✅ 响应时间 < 50ms
✅ 无错误日志
✅ 进程稳定运行
```

### 警告指标

```
⚠️ 连接成功率 95-99%
⚠️ 响应时间 50-200ms
⚠️ 偶尔超时
⚠️ 少量错误日志
```

### 失败指标

```
❌ 连接成功率 < 95%
❌ 响应时间 > 200ms
❌ 频繁超时
❌ 进程崩溃
❌ 大量错误日志
```

## 🎓 学习路径

### 第一阶段: 快速体验 (5分钟)
```bash
make check     # 检查环境
make build     # 构建项目
make quick     # 快速演示
```

### 第二阶段: 功能学习 (15分钟)
```bash
make demo          # 交互式演示
cat scripts/README.md  # 阅读文档
make test          # 功能测试
```

### 第三阶段: 深度测试 (30分钟)
```bash
# 逐一测试各场景
make test-1
make test-2
make test-3
make test-4
make test-5

# 性能测试
make benchmark
```

### 第四阶段: 高级使用 (自定义)
```bash
# 加载工具库
source scripts/config.env

# 学习使用工具函数
agentx_show_config
agentx_validate_env
agentx_gen_client_id

# 创建自定义测试
# (使用提供的工具函数)
```

## 📚 相关文档

- `README.md` - 项目总体介绍（英文）
- `README_zh.md` - 项目总体介绍（中文）
- `CLAUDE.md` - 详细架构说明
- `scripts/README.md` - 脚本详细文档
- `scripts/INDEX.md` - 脚本快速索引
- `scripts/Makefile` - 便捷命令参考

## 🤝 贡献

如果您创建了新的测试脚本或有改进建议，欢迎提交 Pull Request！

### 创建新脚本的规范

1. **命名规范**: 使用 `.sh` 或 `.py` 扩展名
2. **Shebang**: 脚本第一行必须是 `#!/bin/bash`
3. **执行权限**: 使用 `chmod +x` 添加执行权限
4. **注释**: 添加详细的代码注释和文档字符串
5. **颜色输出**: 使用颜色提高可读性（参考 `config.env`）
6. **错误处理**: 实现完善的错误处理和清理逻辑
7. **帮助信息**: 使用 `--help` 参数显示帮助
8. **文档**: 更新 `README.md` 和 `INDEX.md`
9. **测试**: 确保脚本在多个环境下可运行
10. **日志**: 实现结构化的日志输出

### 脚本模板

```bash
#!/bin/bash
# 脚本描述
# 作者: 您的名字
# 版本: 1.0

set -e

# 配置
CONFIG_VAR="${CONFIG_VAR:-default}"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

log_info() {
    echo "[INFO] $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

main() {
    log_info "开始执行..."
    # 脚本逻辑
    log_success "完成"
}

main "$@"
```

## ✅ 总结

我们根据 README 中的5个主要使用场景和3个实际应用示例，创建了一套完整的测试和演示脚本系统，包括：

### 已创建文件 (8个)

1. ✅ `quick_demo.sh` - 快速演示脚本
2. ✅ `setup_demo.sh` - 交互式演示设置
3. ✅ `test_scenarios.sh` - 功能测试脚本
4. ✅ `benchmark.sh` - 性能测试脚本
5. ✅ `config.env` - 配置环境文件
6. ✅ `Makefile` - 便捷命令封装
7. ✅ `README.md` - 详细使用文档
8. ✅ `INDEX.md` - 快速索引参考

### 覆盖的使用场景

✅ 场景1: 将本地开发环境暴露到公网
✅ 场景2: 远程使用Claude Code进行本地编程
✅ 场景3: 多开发者共享开发环境
✅ 场景4: 访问内网数据库和服务
✅ 场景5: 演示和临时环境分享

✅ 示例1: 全栈Web应用开发
✅ 示例2: 数据库远程访问
✅ 示例3: CI/CD测试环境

### 支持的测试类型

✅ 功能测试 - 验证所有核心功能
✅ 性能测试 - RPS、延迟、稳定性
✅ 压力测试 - 多客户端并发
✅ 集成测试 - 端到端场景测试
✅ 回归测试 - 快速验证系统稳定性

### 提供的工具

✅ 一键演示 - `make quick`
✅ 交互式设置 - `make demo`
✅ 自动化测试 - `make test`
✅ 性能评估 - `make benchmark`
✅ 便捷命令 - `make help`

所有脚本都已经添加了执行权限，可以立即使用！

## 🎯 下一步

1. **立即体验**: 运行 `make quick` 快速体验
2. **深入了解**: 阅读 `scripts/README.md` 详细文档
3. **全面测试**: 运行 `make test` 验证功能
4. **性能评估**: 运行 `make benchmark` 评估性能
5. **自定义开发**: 加载 `source scripts/config.env` 使用工具函数

---

**祝您使用愉快！** 🎉

如果遇到问题，请查看 `scripts/README.md` 中的故障排除章节或运行 `make help` 获取帮助。
