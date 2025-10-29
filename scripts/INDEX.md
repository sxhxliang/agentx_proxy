# AgentX Proxy 测试脚本索引

本文档是所有测试和演示脚本的快速参考索引。

## 📚 脚本清单

| 脚本名称 | 类型 | 描述 | 推荐使用场景 |
|---------|------|------|-------------|
| `quick_demo.sh` | 演示 | 快速演示基本功能 | 首次体验 / 5分钟了解 |
| `setup_demo.sh` | 演示 | 交互式演示设置 | 产品演示 / 功能展示 |
| `test_scenarios.sh` | 测试 | 完整功能测试 | 功能验证 / QA测试 |
| `benchmark.sh` | 性能 | 性能基准测试 | 性能评估 / 压力测试 |
| `config.env` | 配置 | 环境配置和工具函数 | 脚本开发 / 高级用户 |
| `README.md` | 文档 | 详细文档和使用说明 | 学习参考 |
| `Makefile` | 工具 | 便捷命令封装 | 快速执行 |

## 🚀 快速开始

### 方法 1: 使用 Make（推荐）

```bash
# 查看所有可用命令
make help

# 检查环境
make check

# 构建项目
make build

# 快速演示
make quick

# 运行所有测试
make all
```

### 方法 2: 直接执行脚本

```bash
# 赋予执行权限
chmod +x scripts/*.sh

# 快速演示
./scripts/quick_demo.sh

# 交互式演示
./scripts/setup_demo.sh

# 功能测试
./scripts/test_scenarios.sh

# 性能测试
./scripts/benchmark.sh
```

### 方法 3: 使用配置环境

```bash
# 加载配置
source scripts/config.env

# 使用配置中的工具函数
agentx_show_config
agentx_validate_env
agentx_gen_client_id

# 启动测试服务
agentx_start_test_service 3000 "My Service"

# 清理环境
agentx_cleanup
```

## 📋 常用命令速查表

### 开发测试

| 任务 | 命令 |
|------|------|
| 检查环境 | `make check` |
| 构建项目 | `make build` |
| 快速演示 | `make quick` |
| 运行所有测试 | `make all` |
| 清理 | `make clean` |

### 特定功能测试

| 场景 | 命令 |
|------|------|
| TCP 代理模式 | `make test-1` |
| 命令模式 | `make test-2` |
| 多客户端 | `make test-3` |
| 数据库访问 | `make test-4` |
| 压力测试 | `make test-5` |

### 性能测试

| 类型 | 命令 |
|------|------|
| 基本性能 | `make benchmark` |
| 稳定性测试 | `make benchmark-stability` |
| 自定义性能 | `make benchmark POOL_SIZE=10 DURATION=60` |

### 演示场景

| 场景 | 命令 |
|------|------|
| Web 开发 | `./scripts/setup_demo.sh -s 1` |
| API 服务 | `./scripts/setup_demo.sh -s 2` |
| 数据库 | `./scripts/setup_demo.sh -s 3` |
| Claude | `./scripts/setup_demo.sh -s 4` |
| 完整演示 | `./scripts/setup_demo.sh -s 5` |

## 🎯 使用场景决策树

```
我需要...
│
├─ 快速体验系统 (5分钟)
│  └─ make quick
│
├─ 了解所有功能
│  └─ make demo (交互式选择)
│
├─ 验证特定功能
│  ├─ TCP 代理 -> make test-1
│  ├─ 命令模式 -> make test-2
│  ├─ 多客户端 -> make test-3
│  ├─ 数据库 -> make test-4
│  └─ 全部 -> make test
│
├─ 性能评估
│  ├─ 基本性能 -> make benchmark
│  ├─ 稳定性 -> make benchmark-stability
│  └─ 自定义 -> POOL_SIZE=10 make benchmark
│
├─ 产品演示
│  └─ ./scripts/setup_demo.sh
│
└─ 开发调试
   ├─ 查看配置 -> source scripts/config.env; agentx_show_config
   ├─ 检查环境 -> make check
   ├─ 查看日志 -> tail -f /tmp/arps.log
   └─ 清理环境 -> make clean
```

## 📊 脚本特性对比

| 特性 | quick_demo | setup_demo | test_scenarios | benchmark |
|------|-----------|-----------|---------------|-----------|
| 自动化程度 | 全自动 | 半自动 | 半自动 | 半自动 |
| 交互性 | 无 | 有 | 有 | 参数配置 |
| 测试覆盖 | 基础 | 中等 | 全面 | 性能专项 |
| 执行时间 | 30秒 | 1-5分钟 | 5-15分钟 | 1-10分钟 |
| 难度等级 | ⭐ | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| 推荐用户 | 新手 | 所有用户 | 开发者 | 性能工程师 |

## 🔧 配置选项

### 环境变量

可以通过环境变量自定义测试行为：

```bash
# 基本配置
export POOL_SIZE=10
export TEST_DURATION=60
export CONCURRENT_USERS=20
export SERVER_HOST="your-server.com"

# 日志配置
export RUST_LOG=debug
export LOG_DIR="/my/logs"

# 运行测试
make benchmark
```

### 命令行参数

```bash
# benchmark.sh 参数
./scripts/benchmark.sh -p 10 -d 60 -c 20

# setup_demo.sh 参数
./scripts/setup_demo.sh -s 1 --pool-size 5 --server-host "0.0.0.0"

# test_scenarios.sh 参数
# (通过交互式选择)
```

## 📁 日志文件位置

所有脚本运行日志保存在 `/tmp/` 目录：

```
/tmp/
├── arps_*.log              # 服务器日志
├── arpc_*.log              # 客户端日志
├── test_server_*.log         # 测试服务日志
├── demo_*.log                # 演示日志
├── benchmark_*.json          # 性能测试结果
└── *.pid                     # 进程 ID 文件
```

## 🐛 常见问题

### Q: 端口被占用怎么办？
A: 脚本会自动处理，或者手动清理：
```bash
lsof -ti:17001 | xargs kill -9
make clean
```

### Q: 测试失败如何调试？
A: 启用详细日志：
```bash
RUST_LOG=debug make quick
tail -f /tmp/arpc_*.log
```

### Q: 性能测试结果在哪里？
A: JSON 格式保存在：
```bash
cat /tmp/agentx_benchmark_*.json
```

### Q: 如何自定义测试服务？
A: 使用 config.env 中的函数：
```bash
source scripts/config.env
agentx_start_test_service 9000 "MyCustomService"
```

## 📞 获取帮助

### 查看完整文档
```bash
cat scripts/README.md
```

### 查看帮助信息
```bash
make help
./scripts/quick_demo.sh --help
./scripts/benchmark.sh --help
./scripts/setup_demo.sh --help
```

### 检查环境
```bash
make check
source scripts/config.env && agentx_validate_env
```

## 🎓 学习路径

1. **新手入门**
   ```bash
   make check      # 检查环境
   make build      # 构建项目
   make quick      # 快速演示
   ```

2. **功能学习**
   ```bash
   make demo       # 交互式演示
   cat scripts/README.md  # 阅读文档
   ```

3. **开发测试**
   ```bash
   make test       # 功能测试
   make benchmark  # 性能测试
   ```

4. **高级使用**
   ```bash
   source scripts/config.env  # 加载工具函数
   # 自定义测试脚本
   ```

## 🤝 贡献指南

创建新脚本时，请：

1. 遵循命名规范：`*.sh` 或 `*.py`
2. 添加 shebang: `#!/bin/bash`
3. 包含详细注释
4. 使用颜色输出
5. 实现错误处理
6. 添加帮助信息
7. 更新此索引文档

## 📝 更新日志

### v1.0 (当前版本)
- ✅ 添加快速演示脚本
- ✅ 添加交互式演示设置
- ✅ 添加完整功能测试
- ✅ 添加性能基准测试
- ✅ 添加配置环境文件
- ✅ 添加 Makefile 便捷命令
- ✅ 添加详细文档

---

**提示**: 所有脚本都支持 Ctrl+C 优雅退出，会自动清理后台进程。
