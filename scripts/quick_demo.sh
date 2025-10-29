#!/bin/bash
# AgentX Proxy 快速演示脚本
# 快速展示基本功能

set -e

# 颜色定义
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

echo ""
echo "========================================"
echo "   AgentX Proxy 快速演示"
echo "========================================"
echo ""

# 检查是否在项目根目录
if [ ! -f "Cargo.toml" ]; then
    echo "错误: 请在项目根目录运行此脚本"
    exit 1
fi

# 启动演示服务
log "步骤 1: 启动测试 HTTP 服务..."
python3 -m http.server 8080 > /tmp/demo_server.log 2>&1 &
DEMO_PID=$!
success "测试服务已启动 (PID: $DEMO_PID)"

sleep 2

# 启动 AgentX 服务器
log "步骤 2: 启动 AgentX 服务器..."
cargo run -p arps -- \
    --control-port 17001 \
    --proxy-port 17002 \
    --public-port 17003 \
    --pool-size 1 \
    > /tmp/arps_demo.log 2>&1 &

AGENT_PID=$!
success "AgentX 服务器已启动 (PID: $AGENT_PID)"

sleep 3

# 启动 AgentX 客户端
log "步骤 3: 启动 AgentX 客户端..."
CLIENT_ID="demo-client-$(date +%s)"
cargo run -p arpc -- \
    --client-id $CLIENT_ID \
    --server-addr 127.0.0.1 \
    --control-port 17001 \
    --proxy-port 17002 \
    --local-addr 127.0.0.1 \
    --local-port 8080 \
    > /tmp/arpc_demo.log 2>&1 &

arpc_PID=$!
success "AgentX 客户端已启动 (PID: $arpc_PID)"
success "客户端 ID: $CLIENT_ID"

sleep 3

# 测试连接
log "步骤 4: 测试代理连接..."
if curl -s "http://localhost:17003/?token=$CLIENT_ID" | grep -q "Directory listing"; then
    success "代理连接测试成功！"
    echo ""
    echo "🎉 演示成功！"
    echo ""
    echo "访问信息:"
    echo "  本地服务地址:  http://localhost:8080"
    echo "  代理访问地址:  http://localhost:17003/?token=$CLIENT_ID"
    echo ""
    echo "您可以通过以下方式访问:"
    echo "  1. 直接访问: http://localhost:8080"
    echo " 2. 通过代理: http://localhost:17003/?token=$CLIENT_ID"
    echo ""
    echo "按 Ctrl+C 停止演示..."
else
    echo "错误: 连接测试失败"
    echo ""
    echo "服务器日志:"
    cat /tmp/arps_demo.log
    echo ""
    echo "客户端日志:"
    cat /tmp/arpc_demo.log
fi

# 等待中断
trap 'echo ""; warning "正在停止服务..."; kill $DEMO_PID 2>/dev/null || true; kill $AGENT_PID 2>/dev/null || true; kill $arpc_PID 2>/dev/null || true; success "服务已停止"; exit 0' INT

# 保持运行
while true; do
    sleep 1
done
