#!/bin/bash
# AgentX Proxy 一键演示环境设置脚本

set -e

# 颜色定义
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
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

error() {
    echo -e "${RED}[✗]${NC} $1"
}

# 配置变量
SERVER_HOST=${SERVER_HOST:-"0.0.0.0"}
CONTROL_PORT=${CONTROL_PORT:-17001}
PROXY_PORT=${PROXY_PORT:-17002}
PUBLIC_PORT=${PUBLIC_PORT:-17003}
POOL_SIZE=${POOL_SIZE:-3}

# 演示场景配置
declare -A DEMO_SCENARIOS

# 场景 1: Web 开发环境
DEMO_SCENARIOS["web-dev"]="将本地 Web 开发服务器暴露到公网"
DEMO_SCENARIOS["web-dev-id"]="web-dev-demo"
DEMO_SCENARIOS["web-dev-local-port"]="3000"

# 场景 2: API 服务
DEMO_SCENARIOS["api-service"]="暴露本地 API 服务"
DEMO_SCENARIOS["api-service-id"]="api-service-demo"
DEMO_SCENARIOS["api-service-local-port"]="8000"

# 场景 3: 数据库
DEMO_SCENARIOS["database"]="暴露本地数据库"
DEMO_SCENARIOS["database-id"]="database-demo"
DEMO_SCENARIOS["database-local-port"]="5432"

# 场景 4: Claude 命令执行
DEMO_SCENARIOS["claude"]="Claude Code 命令执行演示"
DEMO_SCENARIOS["claude-id"]="claude-demo"
DEMO_SCENARIOS["claude-command-mode"]="true"

# 显示横幅
show_banner() {
    clear
    echo ""
    echo "╔════════════════════════════════════════════════════════════╗"
    echo "║                                                            ║"
    echo "║              AgentX Proxy 演示环境设置                     ║"
    echo "║                                                            ║"
    echo "╚════════════════════════════════════════════════════════════╝"
    echo ""
}

# 检查环境
check_environment() {
    log "检查环境..."

    # 检查 Rust
    if ! command -v cargo &> /dev/null; then
        error "Cargo 未安装，请先安装 Rust: https://rustup.rs/"
        exit 1
    fi

    # 检查是否在项目目录
    if [ ! -f "Cargo.toml" ]; then
        error "请在 AgentX Proxy 项目根目录运行此脚本"
        exit 1
    fi

    # 检查项目是否已编译
    if [ ! -d "target/debug" ]; then
        log "首次运行，正在编译项目..."
        cargo build
        success "编译完成"
    fi

    # 检查端口
    for port in $CONTROL_PORT $PROXY_PORT $PUBLIC_PORT; do
        if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
            warning "端口 $port 已被占用，将尝试停止现有进程"
            lsof -ti:$port | xargs kill -9 2>/dev/null || true
            sleep 1
        fi
    done

    success "环境检查通过"
}

# 显示演示场景
show_scenarios() {
    echo ""
    echo "请选择演示场景:"
    echo ""
    echo "  1. 🌐 Web 开发环境演示"
    echo "     将本地开发服务器 (localhost:3000) 暴露到公网"
    echo ""
    echo "  2. 🔌 API 服务演示"
    echo "     暴露本地 REST API 服务 (localhost:8000)"
    echo ""
    echo "  3. 🗄️  数据库访问演示"
    echo "     暴露本地数据库服务 (localhost:5432)"
    echo ""
    echo "  4. 🤖 Claude 命令执行演示"
    echo "     启用 HTTP API 和命令执行模式"
    echo ""
    echo "  5. 🚀 完整演示"
    echo "     运行所有场景"
    echo ""
    echo "  0. 退出"
    echo ""
}

# 创建测试服务
create_test_service() {
    local port=$1
    local name=$2

    log "创建测试服务 '$name' (端口: $port)..."

    # 创建简单的 HTTP 服务器
    cat > /tmp/test_server_$port.py <<EOF
import http.server
import socketserver
import json
from datetime import datetime

class AgentXTestHandler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/health':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            response = {
                "status": "ok",
                "service": "$name",
                "port": $port,
                "time": datetime.now().isoformat()
            }
            self.wfile.write(json.dumps(response, indent=2).encode())
        else:
            super().do_GET()

    def log_message(self, format, *args):
        pass

with socketserver.TCPServer(("127.0.0.1", $port), AgentXTestHandler) as httpd:
    print(f"服务 '$name' 运行在端口 $port")
    httpd.serve_forever()
EOF

    python3 /tmp/test_server_$port.py > /tmp/test_server_$port.log 2>&1 &
    echo $! > /tmp/test_server_$port.pid
    sleep 1

    success "测试服务已启动"
}

# 停止测试服务
stop_test_service() {
    local port=$1
    if [ -f /tmp/test_server_$port.pid ]; then
        kill $(cat /tmp/test_server_$port.pid) 2>/dev/null || true
        rm -f /tmp/test_server_$port.pid
    fi
}

# 启动演示
run_demo() {
    local scenario=$1

    case $scenario in
        1)
            demo_web_dev
            ;;
        2)
            demo_api_service
            ;;
        3)
            demo_database
            ;;
        4)
            demo_claude
            ;;
        5)
            demo_full
            ;;
        *)
            error "无效的场景选择"
            return 1
            ;;
    esac
}

# Web 开发演示
demo_web_dev() {
    log "启动 Web 开发环境演示..."

    # 创建测试服务
    create_test_service 3000 "Web开发服务器"

    # 启动服务器
    log "启动 AgentX 服务器..."
    cargo run -p arps -- \
        --control-port $CONTROL_PORT \
        --proxy-port $PROXY_PORT \
        --public-port $PUBLIC_PORT \
        --pool-size $POOL_SIZE \
        > /tmp/demo_server.log 2>&1 &

    SERVER_PID=$!
    echo $SERVER_PID > /tmp/demo_server.pid
    sleep 3

    # 启动客户端
    log "启动 AgentX 客户端..."
    cargo run -p arpc -- \
        --client-id web-dev-demo \
        --server-addr $SERVER_HOST \
        --control-port $CONTROL_PORT \
        --proxy-port $PROXY_PORT \
        --local-addr 127.0.0.1 \
        --local-port 3000 \
        > /tmp/demo_client.log 2>&1 &

    CLIENT_PID=$!
    echo $CLIENT_PID > /tmp/demo_client.pid
    sleep 3

    success "演示已启动！"
    echo ""
    echo "访问信息:"
    echo "  本地服务: http://localhost:3000"
    echo "  代理地址: http://localhost:$PUBLIC_PORT/?token=web-dev-demo"
    echo ""
    echo "按 Ctrl+C 停止演示"
    echo ""

    trap 'stop_demo $CLIENT_PID $SERVER_PID 3000' INT
    wait
}

# API 服务演示
demo_api_service() {
    log "启动 API 服务演示..."

    # 创建 API 测试服务
    create_test_service 8000 "API服务"

    # 启动服务器和客户端（类似上面）
    cargo run -p arps -- \
        --control-port $CONTROL_PORT \
        --proxy-port $PROXY_PORT \
        --public-port $PUBLIC_PORT \
        --pool-size $POOL_SIZE \
        > /tmp/demo_server.log 2>&1 &

    SERVER_PID=$!
    echo $SERVER_PID > /tmp/demo_server.pid
    sleep 3

    cargo run -p arpc -- \
        --client-id api-service-demo \
        --server-addr $SERVER_HOST \
        --control-port $CONTROL_PORT \
        --proxy-port $PROXY_PORT \
        --local-addr 127.0.0.1 \
        --local-port 8000 \
        > /tmp/demo_client.log 2>&1 &

    CLIENT_PID=$!
    echo $CLIENT_PID > /tmp/demo_client.pid
    sleep 3

    success "演示已启动！"
    echo ""
    echo "API 访问:"
    echo "  直接访问: http://localhost:8000"
    echo "  通过代理: http://localhost:$PUBLIC_PORT/?token=api-service-demo"
    echo ""

    trap 'stop_demo $CLIENT_PID $SERVER_PID 8000' INT
    wait
}

# 数据库演示
demo_database() {
    log "启动数据库访问演示..."

    # 模拟数据库
    log "启动模拟 PostgreSQL 数据库..."
    nc -l -p 5432 > /tmp/db_mock.log 2>&1 &
    DB_PID=$!
    sleep 1

    # 启动服务器和客户端
    cargo run -p arps -- \
        --control-port $CONTROL_PORT \
        --proxy-port $PROXY_PORT \
        --public-port $PUBLIC_PORT \
        --pool-size $POOL_SIZE \
        > /tmp/demo_server.log 2>&1 &

    SERVER_PID=$!
    echo $SERVER_PID > /tmp/demo_server.pid
    sleep 3

    cargo run -p arpc -- \
        --client-id database-demo \
        --server-addr $SERVER_HOST \
        --control-port $CONTROL_PORT \
        --proxy-port $PROXY_PORT \
        --local-addr 127.0.0.1 \
        --local-port 5432 \
        > /tmp/demo_client.log 2>&1 &

    CLIENT_PID=$!
    echo $CLIENT_PID > /tmp/demo_client.pid
    sleep 3

    success "演示已启动！"
    echo ""
    echo "数据库连接:"
    echo "  psql -h localhost -p $PUBLIC_PORT -d postgres ?token=database-demo"
    echo "  mysql -h localhost -P $PUBLIC_PORT -u root -p ?token=database-demo"
    echo ""

    trap 'stop_demo_db $CLIENT_PID $SERVER_PID $DB_PID' INT
    wait
}

# Claude 演示
demo_claude() {
    log "启动 Claude 命令执行演示..."

    # 启动服务器
    cargo run -p arps -- \
        --control-port $CONTROL_PORT \
        --proxy-port $PROXY_PORT \
        --public-port $PUBLIC_PORT \
        --pool-size $POOL_SIZE \
        > /tmp/demo_server.log 2>&1 &

    SERVER_PID=$!
    echo $SERVER_PID > /tmp/demo_server.pid
    sleep 3

    # 启动客户端（命令模式）
    log "启动 AgentX 客户端（命令模式）..."
    cargo run -p arpc -- \
        --client-id claude-demo \
        --server-addr $SERVER_HOST \
        --control-port $CONTROL_PORT \
        --proxy-port $PROXY_PORT \
        --command-mode \
        > /tmp/demo_client.log 2>&1 &

    CLIENT_PID=$!
    echo $CLIENT_PID > /tmp/demo_client.pid
    sleep 3

    success "演示已启动！"
    echo ""
    echo "Claude 命令执行:"
    echo "  代理地址: http://localhost:$PUBLIC_PORT/?token=claude-demo"
    echo ""
    echo "API 端点:"
    echo "  POST /api/sessions - 创建新会话"
    echo "  GET /api/sessions/{id} - 获取会话状态"
    echo ""
    echo "示例请求:"
    echo '  curl -X POST "http://localhost:17003/?token=claude-demo/sessions" \'
    echo '       -H "Content-Type: application/json" \'
    echo '       -d '"'"'{"executor": "claude", "prompt": "echo Hello"}'"'"
    echo ""

    trap 'stop_demo $CLIENT_PID $SERVER_PID' INT
    wait
}

# 完整演示
demo_full() {
    log "运行完整演示场景..."

    # 运行所有场景
    for i in {1..4}; do
        echo ""
        echo "==================== 场景 $i ===================="
        run_demo $i
        echo ""

        if [ $i -lt 4 ]; then
            log "等待 5 秒后继续下一个场景..."
            sleep 5
        fi
    done

    success "完整演示完成！"
}

# 停止演示
stop_demo() {
    local client_pid=$1
    local server_pid=$2
    local local_port=$3

    echo ""
    warning "正在停止演示..."

    if [ ! -z "$client_pid" ]; then
        kill $client_pid 2>/dev/null || true
    fi

    if [ ! -z "$server_pid" ]; then
        kill $server_pid 2>/dev/null || true
    fi

    if [ ! -z "$local_port" ]; then
        stop_test_service $local_port
    fi

    pkill -f "cargo run -p arps" 2>/dev/null || true
    pkill -f "arpc" 2>/dev/null || true

    success "演示已停止"
    exit 0
}

stop_demo_db() {
    local client_pid=$1
    local server_pid=$2
    local db_pid=$3

    echo ""
    warning "正在停止演示..."

    kill $client_pid 2>/dev/null || true
    kill $server_pid 2>/dev/null || true
    kill $db_pid 2>/dev/null || true

    pkill -f "cargo run -p arps" 2>/dev/null || true
    pkill -f "arpc" 2>/dev/null || true

    success "演示已停止"
    exit 0
}

# 显示帮助
show_help() {
    cat <<EOF
AgentX Proxy 演示环境设置脚本

用法:
  $0 [选项]

选项:
  -h, --help              显示此帮助信息
  -s, --scenario NUM      直接运行指定场景 (1-5)
  --pool-size SIZE        设置连接池大小
  --server-host HOST      设置服务器地址 (默认: 0.0.0.0)
  --ports CTRL:PROXY:PUB  设置端口 (默认: 17001:17002:17003)

场景编号:
  1 - Web 开发环境
  2 - API 服务
  3 - 数据库访问
  4 - Claude 命令执行
  5 - 完整演示

环境变量:
  SERVER_HOST             服务器地址
  CONTROL_PORT            控制端口
  PROXY_PORT              代理端口
  PUBLIC_PORT             公网端口
  POOL_SIZE               连接池大小

示例:
  $0                      # 交互式选择场景
  $0 -s 1                 # 直接运行 Web 开发演示
  $0 -s 4 --pool-size 5   # 运行 Claude 演示，连接池大小为 5
  POOL_SIZE=10 $0 -s 2    # 运行 API 演示，使用环境变量设置连接池

EOF
}

# 主函数
main() {
    local scenario=""

    # 解析参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -s|--scenario)
                scenario="$2"
                shift 2
                ;;
            --pool-size)
                POOL_SIZE="$2"
                shift 2
                ;;
            --server-host)
                SERVER_HOST="$2"
                shift 2
                ;;
            --ports)
                IFS=':' read -r CONTROL_PORT PROXY_PORT PUBLIC_PORT <<< "$2"
                shift 2
                ;;
            *)
                error "未知参数: $1"
                show_help
                exit 1
                ;;
        esac
    done

    show_banner
    check_environment

    # 如果指定了场景，直接运行
    if [ ! -z "$scenario" ]; then
        run_demo $scenario
        exit 0
    fi

    # 否则显示交互式菜单
    show_scenarios

    while true; do
        read -p "请输入选择 (0-5): " choice

        case $choice in
            0)
                success "退出"
                exit 0
                ;;
            1|2|3|4|5)
                run_demo $choice
                break
                ;;
            *)
                error "无效选择，请输入 0-5"
                ;;
        esac
    done
}

# 运行主函数
main "$@"
