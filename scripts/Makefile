# AgentX Proxy 测试脚本 Makefile
# 提供便捷的测试命令

.PHONY: help demo test benchmark clean all setup quick

# 默认目标
help:
	@echo "AgentX Proxy 测试脚本"
	@echo ""
	@echo "可用命令:"
	@echo "  make quick       - 运行快速演示 (推荐新用户)"
	@echo "  make demo        - 运行交互式演示设置"
	@echo "  make test        - 运行完整功能测试"
	@echo "  make test-1      - 测试场景 1: TCP 代理模式"
	@echo "  make test-2      - 测试场景 2: 命令模式"
	@echo "  make test-3      - 测试场景 3: 多客户端"
	@echo "  make test-4      - 测试场景 4: 数据库"
	@echo "  make test-5      - 测试场景 5: 压力测试"
	@echo "  make benchmark   - 运行性能基准测试"
	@echo "  make benchmark-stability - 稳定性测试"
	@echo "  make clean       - 清理所有临时文件和进程"
	@echo "  make all         - 运行所有测试和演示"
	@echo ""
	@echo "高级选项:"
	@echo "  make benchmark POOL_SIZE=10 DURATION=60 - 自定义性能测试"
	@echo "  make setup SCENARIO=4 - 运行特定演示场景"
	@echo ""
	@echo "更多信息请查看: scripts/README.md"

# 快速演示
quick:
	@echo "运行快速演示..."
	@chmod +x scripts/quick_demo.sh
	@./scripts/quick_demo.sh

# 交互式演示设置
demo:
	@echo "启动交互式演示设置..."
	@chmod +x scripts/setup_demo.sh
	@./scripts/setup_demo.sh

# 演示特定场景
setup:
	@chmod +x scripts/setup_demo.sh
	@./scripts/setup_demo.sh -s $(SCENARIO)

# 完整功能测试
test:
	@echo "运行完整功能测试..."
	@chmod +x scripts/test_scenarios.sh
	@./scripts/test_scenarios.sh <<< "6"

# 特定场景测试
test-1:
	@echo "测试场景 1: TCP 代理模式"
	@chmod +x scripts/test_scenarios.sh
	@./scripts/test_scenarios.sh <<< "1"

test-2:
	@echo "测试场景 2: 命令模式"
	@chmod +x scripts/test_scenarios.sh
	@./scripts/test_scenarios.sh <<< "2"

test-3:
	@echo "测试场景 3: 多客户端"
	@chmod +x scripts/test_scenarios.sh
	@./scripts/test_scenarios.sh <<< "3"

test-4:
	@echo "测试场景 4: 数据库"
	@chmod +x scripts/test_scenarios.sh
	@./scripts/test_scenarios.sh <<< "4"

test-5:
	@echo "测试场景 5: 压力测试"
	@chmod +x scripts/test_scenarios.sh
	@./scripts/test_scenarios.sh <<< "5"

# 性能基准测试
benchmark:
	@echo "运行性能基准测试..."
	@chmod +x scripts/benchmark.sh
	@POOL_SIZE=$(POOL_SIZE) TEST_DURATION=$(DURATION) CONCURRENT_USERS=$(CONCURRENT) ./scripts/benchmark.sh

# 稳定性测试
benchmark-stability:
	@echo "运行稳定性测试..."
	@chmod +x scripts/benchmark.sh
	@./scripts/benchmark.sh --stability -d $(DURATION)

# 清理
clean:
	@echo "清理临时文件和进程..."
	@pkill -f "cargo run -p arps" 2>/dev/null || true
	@pkill -f "arpc" 2>/dev/null || true
	@pkill -f "python3 -m http.server" 2>/dev/null || true
	@pkill -f "python3 /tmp/test_" 2>/dev/null || true
	@pkill -f "nc -l" 2>/dev/null || true
	@rm -f /tmp/arps_*.log /tmp/arpc_*.log
	@rm -f /tmp/test_*.log /tmp/test_*.pid
	@rm -f /tmp/demo_*.log /tmp/demo_*.pid
	@rm -f /tmp/benchmark_*.json
	@rm -f /tmp/agent_*.pid /tmp/arpc_*.pid
	@rm -f /tmp/db_mock.pid
	@rm -f /tmp/*.py
	@echo "清理完成"

# 运行所有测试
all: clean
	@echo "运行所有测试..."
	@$(MAKE) quick || true
	@$(MAKE) test || true
	@$(MAKE) benchmark || true
	@echo ""
	@echo "所有测试完成!"

# 检查环境
check:
	@echo "检查环境..."
	@which cargo >/dev/null && echo "✅ Cargo 已安装" || (echo "❌ Cargo 未安装" && exit 1)
	@which curl >/dev/null && echo "✅ curl 已安装" || echo "⚠️  curl 未安装 (某些测试可能失败)"
	@which python3 >/dev/null && echo "✅ Python3 已安装" || echo "⚠️  Python3 未安装 (某些测试可能失败)"
	@which bc >/dev/null && echo "✅ bc 已安装" || echo "⚠️  bc 未安装 (性能测试可能失败)"
	@echo ""
	@cargo --version && echo ""
	@cargo check --workspace 2>/dev/null && echo "✅ 项目可以编译" || echo "❌ 项目编译失败"

# 构建项目
build:
	@echo "构建项目..."
	@cargo build --workspace
	@echo "构建完成"

# 构建发布版本
build-release:
	@echo "构建发布版本..."
	@cargo build --release --workspace
	@echo "构建完成"

# 安装脚本执行权限
install:
	@chmod +x scripts/*.sh
	@echo "脚本权限设置完成"

# 显示项目信息
info:
	@echo "AgentX Proxy 项目信息"
	@echo "======================"
	@echo ""
	@echo "项目结构:"
	@echo "  arp-server/  - 服务器组件"
	@echo "  arp-client/  - 客户端组件"
	@echo "  arp-common/  - 共享协议库"
	@echo ""
	@echo "默认端口:"
	@echo "  控制端口: 17001"
	@echo "  代理端口: 17002"
	@echo "  公网端口: 17003"
	@echo ""
	@echo "可用脚本:"
	@ls -1 scripts/*.sh | sed 's|scripts/||' | sed 's|\.sh||' | awk '{print "  - " $$0}'
	@echo ""
	@echo "快速开始:"
	@echo "  1. make check     - 检查环境"
	@echo "  2. make build     - 构建项目"
	@echo "  3. make quick     - 快速演示"
	@echo ""
