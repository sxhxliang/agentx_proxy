#!/bin/bash
# AgentX Proxy 测试场景脚本
# 用于演示各种使用场景

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置
SERVER_ADDR="127.0.0.1"
CONTROL_PORT=17001
PROXY_PORT=17002
PUBLIC_PORT=17003
POOL_SIZE=2

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查依赖
check_dependencies() {
    log_info "检查依赖..."

    if ! command -v cargo &> /dev/null; then
        log_error "Cargo 未安装，请先安装 Rust"
        exit 1
    fi

    if ! command -v curl &> /dev/null; then
        log_error "curl 未安装"
        exit 1
    fi

    # 检查端口是否被占用
    if lsof -Pi :$CONTROL_PORT -sTCP:LISTEN -t >/dev/null 2>&1; then
        log_warning "端口 $CONTROL_PORT 已被占用"
    fi

    if lsof -Pi :$PROXY_PORT -sTCP:LISTEN -t >/dev/null 2>&1; then
        log_warning "端口 $PROXY_PORT 已被占用"
    fi

    if lsof -Pi :$PUBLIC_PORT -sTCP:LISTEN -t >/dev/null 2>&1; then
        log_warning "端口 $PUBLIC_PORT 已被占用"
    fi

    log_success "依赖检查完成"
}

# 启动服务器
start_server() {
    log_info "启动 AgentX 服务器..."

    # 杀死可能存在的旧进程
    pkill -f "cargo run -p arps" || true
    sleep 2

    # 启动服务器
    cargo run -p arps -- \
        --control-port $CONTROL_PORT \
        --proxy-port $PROXY_PORT \
        --public-port $PUBLIC_PORT \
        --pool-size $POOL_SIZE \
        > /tmp/arps.log 2>&1 &

    AGENT_PID=$!
    echo $AGENT_PID > /tmp/arps.pid

    log_info "服务器已启动，PID: $AGENT_PID"
    log_info "等待服务器启动..."

    # 等待服务器启动
    for i in {1..10}; do
        if curl -s http://localhost:$CONTROL_PORT >/dev/null 2>&1; then
            break
        fi
        sleep 1
    done

    sleep 2
    log_success "服务器启动完成"
}

# 停止服务器
stop_server() {
    if [ -f /tmp/arps.pid ]; then
        AGENT_PID=$(cat /tmp/arps.pid)
        if kill -0 $AGENT_PID 2>/dev/null; then
            log_info "停止服务器 (PID: $AGENT_PID)..."
            kill $AGENT_PID
            sleep 2
        fi
        rm -f /tmp/arps.pid
    fi
    pkill -f "cargo run -p arps" || true
}

# 启动本地测试服务
start_test_service() {
    local port=$1
    local name=$2

    log_info "启动测试服务 '$name' 在端口 $port..."

    # 创建一个简单的 HTTP 服务
    python3 -m http.server $port > /tmp/test_service_$port.log 2>&1 &
    SERVICE_PID=$!
    echo $SERVICE_PID > /tmp/test_service_$port.pid

    sleep 2
    log_success "测试服务 '$name' 已启动，PID: $SERVICE_PID"
}

# 停止测试服务
stop_test_service() {
    local port=$1
    if [ -f /tmp/test_service_$port.pid ]; then
        SERVICE_PID=$(cat /tmp/test_service_$port.pid)
        if kill -0 $SERVICE_PID 2>/dev/null; then
            kill $SERVICE_PID
            sleep 1
        fi
        rm -f /tmp/test_service_$port.pid
    fi
}

# 场景1: TCP 代理模式测试
scenario_tcp_proxy() {
    log_info "=== 场景 1: TCP 代理模式 - 将本地开发环境暴露到公网 ==="

    local client_id="test-tcp-proxy"
    local local_port=8000

    # 启动测试服务
    start_test_service $local_port "Test Service"

    # 启动客户端
    log_info "启动 AgentX 客户端..."
    cargo run -p arpc -- \
        --client-id $client_id \
        --server-addr $SERVER_ADDR \
        --control-port $CONTROL_PORT \
        --proxy-port $PROXY_PORT \
        --local-addr 127.0.0.1 \
        --local-port $local_port \
        > /tmp/arpc_$client_id.log 2>&1 &

    arpc_PID=$!
    echo $arpc_PID > /tmp/arpc_$client_id.pid

    log_info "等待客户端连接..."
    sleep 3

    # 测试连接
    log_info "测试代理连接..."
    if curl -s "http://localhost:$PUBLIC_PORT/?token=$client_id" | grep -q "Directory listing"; then
        log_success "TCP 代理测试成功！"
        echo "访问地址: http://localhost:$PUBLIC_PORT/?token=$client_id"
    else
        log_error "TCP 代理测试失败"
    fi

    # 清理
    if [ -f /tmp/arpc_$client_id.pid ]; then
        kill $(cat /tmp/arpc_$client_id.pid) 2>/dev/null || true
    fi
    stop_test_service $local_port
}

# 场景2: 命令模式测试
scenario_command_mode() {
    log_info "=== 场景 2: 命令模式 - HTTP API 测试 ==="

    local client_id="test-command-mode"

    # 启动客户端（命令模式）
    log_info "启动 AgentX 客户端（命令模式）..."
    cargo run -p arpc -- \
        --client-id $client_id \
        --server-addr $SERVER_ADDR \
        --control-port $CONTROL_PORT \
        --proxy-port $PROXY_PORT \
        --command-mode \
        > /tmp/arpc_$client_id.log 2>&1 &

    arpc_PID=$!
    echo $arpc_PID > /tmp/arpc_$client_id.pid

    log_info "等待客户端启动..."
    sleep 3

    # 测试 API
    log_info "测试会话创建 API..."
    response=$(curl -s -X POST "http://localhost:$PUBLIC_PORT/?token=$client_id/sessions" \
        -H "Content-Type: application/json" \
        -d '{"executor": "claude", "prompt": "echo Hello World"}')

    if echo "$response" | grep -q "session_id"; then
        log_success "会话创建 API 测试成功！"
        session_id=$(echo "$response" | grep -o '"session_id":"[^"]*"' | cut -d'"' -f4)
        echo "会话 ID: $session_id"

        # 测试获取会话
        sleep 2
        session_info=$(curl -s "http://localhost:$PUBLIC_PORT/?token=$client_id/sessions/$session_id")
        log_success "获取会话信息成功"
    else
        log_error "命令模式 API 测试失败"
        cat /tmp/arpc_$client_id.log
    fi

    # 清理
    if [ -f /tmp/arpc_$client_id.pid ]; then
        kill $(cat /tmp/arpc_$client_id.pid) 2>/dev/null || true
    fi
}

# 场景3: 多客户端测试
scenario_multiple_clients() {
    log_info "=== 场景 3: 多开发者共享开发环境 ==="

    # 启动多个客户端
    for i in {1..3}; do
        local client_id="user$i"
        local local_port=$((8000 + i))

        start_test_service $local_port "User$i Service"

        cargo run -p arpc -- \
            --client-id $client_id \
            --server-addr $SERVER_ADDR \
            --control-port $CONTROL_PORT \
            --proxy-port $PROXY_PORT \
            --local-addr 127.0.0.1 \
            --local-port $local_port \
            > /tmp/arpc_$client_id.log 2>&1 &

        echo $! > /tmp/arpc_$client_id.pid
        log_info "启动用户 $i 的客户端 (client_id: $client_id, port: $local_port)"
    done

    sleep 3

    # 测试所有客户端
    for i in {1..3}; do
        local client_id="user$i"
        log_info "测试用户 $i 的连接..."

        if curl -s "http://localhost:$PUBLIC_PORT/?token=$client_id" >/dev/null 2>&1; then
            log_success "用户 $i 连接成功"
            echo "访问地址: http://localhost:$PUBLIC_PORT/?token=$client_id"
        else
            log_warning "用户 $i 连接可能失败"
        fi
    done

    # 清理
    for i in {1..3}; do
        local client_id="user$i"
        if [ -f /tmp/arpc_$client_id.pid ]; then
            kill $(cat /tmp/arpc_$client_id.pid) 2>/dev/null || true
        fi
        stop_test_service $((8000 + i))
    done
}

# 场景4: 数据库模拟测试
scenario_database() {
    log_info "=== 场景 4: 访问内网数据库（模拟） ==="

    local client_id="test-database"

    # 使用 nc 模拟数据库监听
    log_info "启动模拟数据库服务 (端口 5432)..."
    nc -l -p 5432 > /tmp/db_mock.log 2>&1 &
    DB_PID=$!
    echo $DB_PID > /tmp/db_mock.pid

    # 启动客户端
    log_info "启动 AgentX 客户端..."
    cargo run -p arpc -- \
        --client-id $client_id \
        --server-addr $SERVER_ADDR \
        --control-port $CONTROL_PORT \
        --proxy-port $PROXY_PORT \
        --local-addr 127.0.0.1 \
        --local-port 5432 \
        > /tmp/arpc_$client_id.log 2>&1 &

    arpc_PID=$!
    echo $arpc_PID > /tmp/arpc_$client_id.pid

    sleep 3

    # 测试连接
    log_info "测试数据库连接..."
    if curl -s "http://localhost:$PUBLIC_PORT/?token=$client_id" >/dev/null 2>&1; then
        log_success "数据库代理测试成功！"
        echo "连接命令: psql -h localhost -p $PUBLIC_PORT -d postgres ?token=$client_id"
    else
        log_error "数据库代理测试失败"
    fi

    # 清理
    if [ -f /tmp/arpc_$client_id.pid ]; then
        kill $(cat /tmp/arpc_$client_id.pid) 2>/dev/null || true
    fi
    if [ -f /tmp/db_mock.pid ]; then
        kill $(cat /tmp/db_mock.pid) 2>/dev/null || true
    fi
}

# 场景5: 压力测试
scenario_load_test() {
    log_info "=== 场景 5: 连接池压力测试 ==="

    local client_id="test-load"
    local local_port=8888

    start_test_service $local_port "Load Test Service"

    # 启动多个客户端实例
    for i in {1..5}; do
        cargo run -p arpc -- \
            --client-id ${client_id}_$i \
            --server-addr $SERVER_ADDR \
            --control-port $CONTROL_PORT \
            --proxy-port $PROXY_PORT \
            --local-addr 127.0.0.1 \
            --local-port $local_port \
            > /tmp/arpc_load_$i.log 2>&1 &

        echo $! > /tmp/arpc_load_$i.pid
    done

    sleep 5

    # 并发测试
    log_info "运行并发测试（10 个请求）..."
    for i in {1..10}; do
        curl -s "http://localhost:$PUBLIC_PORT/?token=${client_id}_$((i % 5 + 1))" >/dev/null &
    done

    wait
    log_success "并发测试完成"

    # 清理
    for i in {1..5}; do
        if [ -f /tmp/arpc_load_$i.pid ]; then
            kill $(cat /tmp/arpc_load_$i.pid) 2>/dev/null || true
        fi
    done
    stop_test_service $local_port
}

# 主函数
main() {
    echo "========================================"
    echo "  AgentX Proxy 测试场景脚本"
    echo "========================================"
    echo ""

    # 清理之前的进程
    stop_server
    pkill -f "python3 -m http.server" || true
    sleep 2

    # 检查依赖
    check_dependencies

    # 启动服务器
    start_server

    echo ""
    log_info "选择要运行的测试场景:"
    echo "1) 场景 1: TCP 代理模式"
    echo "2) 场景 2: 命令模式"
    echo "3) 场景 3: 多客户端"
    echo "4) 场景 4: 数据库模拟"
    echo "5) 场景 5: 压力测试"
    echo "6) 运行所有场景"
    echo ""

    read -p "请输入选择 (1-6): " choice

    case $choice in
        1)
            scenario_tcp_proxy
            ;;
        2)
            scenario_command_mode
            ;;
        3)
            scenario_multiple_clients
            ;;
        4)
            scenario_database
            ;;
        5)
            scenario_load_test
            ;;
        6)
            scenario_tcp_proxy
            echo ""
            scenario_command_mode
            echo ""
            scenario_multiple_clients
            echo ""
            scenario_database
            echo ""
            scenario_load_test
            ;;
        *)
            log_error "无效的选择"
            exit 1
            ;;
    esac

    echo ""
    log_info "清理环境..."
    stop_server
    pkill -f "python3 -m http.server" || true
    pkill -f "nc -l" || true

    log_success "测试完成！"
}

# 捕获 Ctrl+C
trap 'log_warning "接收到中断信号，正在清理..."; stop_server; pkill -f "python3 -m http.server" || true; pkill -f "nc -l" || true; exit 0' INT

# 运行主函数
main "$@"
