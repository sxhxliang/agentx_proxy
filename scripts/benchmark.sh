#!/bin/bash
# AgentX Proxy 性能测试脚本

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 配置
CONTROL_PORT=17001
PROXY_PORT=17002
PUBLIC_PORT=17003
POOL_SIZE=${POOL_SIZE:-5}
TEST_DURATION=${TEST_DURATION:-30}
CONCURRENT_USERS=${CONCURRENT_USERS:-10}

# 结果文件
RESULTS_FILE="/tmp/agentx_benchmark_$(date +%s).json"

# 清理函数
cleanup() {
    log "清理进程..."
    pkill -f "cargo run -p agents" 2>/dev/null || true
    pkill -f "python3 -m http.server" 2>/dev/null || true
    pkill -f "agentc" 2>/dev/null || true
    sleep 2
}

# 启动服务器
start_server() {
    log "启动 AgentX 服务器 (连接池大小: $POOL_SIZE)..."

    cargo run -p agents -- \
        --control-port $CONTROL_PORT \
        --proxy-port $PROXY_PORT \
        --public-port $PUBLIC_PORT \
        --pool-size $POOL_SIZE \
        > /tmp/agents_benchmark.log 2>&1 &

    AGENT_PID=$!
    echo $AGENT_PID > /tmp/agent_pid

    # 等待启动
    for i in {1..10}; do
        if curl -s http://localhost:$CONTROL_PORT >/dev/null 2>&1; then
            break
        fi
        sleep 1
    done

    success "服务器已启动 (PID: $AGENT_PID)"
}

# 启动测试服务
start_test_service() {
    log "启动测试 HTTP 服务..."

    # 创建一个带有延迟的测试服务
    cat > /tmp/test_handler.py <<'EOF'
from http.server import HTTPServer, BaseHTTPRequestHandler
import time

class TestHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        # 模拟处理时间
        time.sleep(0.01)
        self.send_response(200)
        self.send_header('Content-type', 'text/html')
        self.end_headers()
        response = f"""
        <html>
        <body>
            <h1>AgentX Proxy Test Server</h1>
            <p>Request from: {self.client_addr[0]}</p>
            <p>Path: {self.path}</p>
            <p>Time: {time.time()}</p>
        </body>
        </html>
        """.encode()
        self.wfile.write(response)

    def log_message(self, format, *args):
        pass  # 禁用日志

if __name__ == '__main__':
    server = HTTPServer(('127.0.0.1', 9000), TestHandler)
    print("Test server running on port 9000")
    server.serve_forever()
EOF

    python3 /tmp/test_handler.py > /tmp/test_server.log 2>&1 &
    TEST_PID=$!
    echo $TEST_PID > /tmp/test_server_pid
    sleep 2
    success "测试服务已启动 (PID: $TEST_PID)"
}

# 启动多个客户端
start_clients() {
    local num_clients=$1
    log "启动 $num_clients 个客户端实例..."

    for i in $(seq 1 $num_clients); do
        local client_id="benchmark-client-$i"

        cargo run -p agentc -- \
            --client-id $client_id \
            --server-addr 127.0.0.1 \
            --control-port $CONTROL_PORT \
            --proxy-port $PROXY_PORT \
            --local-addr 127.0.0.1 \
            --local-port 9000 \
            > /tmp/agentc_$i.log 2>&1 &

        echo $! > /tmp/agentc_$i.pid
    done

    sleep 5
    success "客户端已启动"
}

# 并发测试
run_concurrent_test() {
    local num_requests=$1
    local client_id=$2

    log "运行并发测试 (请求数: $num_requests)..."

    local start_time=$(date +%s.%N)
    local successful=0
    local failed=0

    # 并发请求
    for i in $(seq 1 $num_requests); do
        (
            if curl -s -w "%{http_code}" -o /dev/null \
                "http://localhost:$PUBLIC_PORT/?token=$client_id" > /tmp/response_$i.txt 2>&1; then
                echo "success" > /tmp/result_$i.txt
            else
                echo "failed" > /tmp/result_$i.txt
            fi
        ) &
    done

    # 等待所有请求完成
    wait

    # 统计结果
    for i in $(seq 1 $num_requests); do
        if [ -f /tmp/result_$i.txt ]; then
            if grep -q "success" /tmp/result_$i.txt; then
                ((successful++))
            else
                ((failed++))
            fi
            rm -f /tmp/result_$i.txt /tmp/response_$i.txt
        fi
    done

    local end_time=$(date +%s.%N)
    local duration=$(echo "$end_time - $start_time" | bc)
    local rps=$(echo "scale=2; $successful / $duration" | bc)

    echo ""
    log "测试结果:"
    echo "  总请求数: $num_requests"
    echo "  成功: $successful"
    echo "  失败: $failed"
    echo "  耗时: ${duration}s"
    echo "  平均 RPS: $rps"
    echo ""

    # 保存结果
    cat > $RESULTS_FILE <<EOF
{
    "test_type": "concurrent",
    "num_requests": $num_requests,
    "successful": $successful,
    "failed": $failed,
    "duration": $duration,
    "rps": $rps,
    "pool_size": $POOL_SIZE,
    "timestamp": "$(date -Iseconds)"
}
EOF
}

# 连接池测试
test_connection_pool() {
    log "测试连接池性能..."

    # 测试连接池为空时的性能
    log "测试 1: 无预热连接池..."
    cargo run -p agents -- \
        --control-port 17011 \
        --proxy-port 17012 \
        --public-port 17013 \
        --pool-size 0 \
        > /tmp/agents_cold.log 2>&1 &

    COLD_PID=$!
    sleep 3

    # 启动客户端
    cargo run -p agentc -- \
        --client-id cold-pool-test \
        --server-addr 127.0.0.1 \
        --control-port 17011 \
        --proxy-port 17012 \
        --local-addr 127.0.0.1 \
        --local-port 9000 \
        > /tmp/agentc_cold.log 2>&1 &

    sleep 5

    local start_time=$(date +%s%3N)
    curl -s "http://localhost:17013/?token=cold-pool-test" > /dev/null
    local end_time=$(date +%s%3N)
    local cold_time=$((end_time - start_time))

    kill $COLD_PID 2>/dev/null || true
    pkill -f "agentc" 2>/dev/null || true
    sleep 2

    log "冷启动延迟: ${cold_time}ms"

    # 测试预热连接池
    log "测试 2: 预热连接池..."
    cargo run -p agents -- \
        --control-port 17011 \
        --proxy-port 17012 \
        --public-port 17013 \
        --pool-size 5 \
        > /tmp/agents_warm.log 2>&1 &

    WARM_PID=$!
    sleep 3

    # 启动客户端
    cargo run -p agentc -- \
        --client-id warm-pool-test \
        --server-addr 127.0.0.1 \
        --control-port 17011 \
        --proxy-port 17012 \
        --local-addr 127.0.0.1 \
        --local-port 9000 \
        > /tmp/agentc_warm.log 2>&1 &

    sleep 8  # 等待连接池建立

    start_time=$(date +%s%3N)
    curl -s "http://localhost:17013/?token=warm-pool-test" > /dev/null
    end_time=$(date +%s%3N)
    local warm_time=$((end_time - start_time))

    kill $WARM_PID 2>/dev/null || true
    pkill -f "agentc" 2>/dev/null || true

    log "预热启动延迟: ${warm_time}ms"

    if [ $cold_time -gt 0 ] && [ $warm_time -gt 0 ]; then
        local improvement=$(echo "scale=2; ($cold_time - $warm_time) * 100 / $cold_time" | bc)
        success "连接池优化效果: ${improvement}% 延迟降低"
    fi
}

# 长期稳定性测试
run_stability_test() {
    local duration=$1

    log "运行稳定性测试 (持续时间: ${duration}s)..."

    local start_time=$(date +%s)
    local request_count=0
    local error_count=0

    while [ $(($(date +%s) - start_time)) -lt $duration ]; do
        for i in {1..5}; do
            local client_id="stability-client-$((i % 5 + 1))"
            if ! curl -s "http://localhost:$PUBLIC_PORT/?token=$client_id" > /dev/null 2>&1; then
                ((error_count++))
            fi
            ((request_count++))
        done
        sleep 1
    done

    local success_count=$((request_count - error_count))
    local success_rate=$(echo "scale=2; $success_count * 100 / $request_count" | bc)

    log "稳定性测试结果:"
    echo "  总请求: $request_count"
    echo "  成功: $success_count"
    echo "  失败: $error_count"
    echo "  成功率: ${success_rate}%"
    echo ""

    cat >> $RESULTS_FILE <<EOF
,
{
    "test_type": "stability",
    "duration": $duration,
    "request_count": $request_count,
    "error_count": $error_count,
    "success_rate": $success_rate
}
EOF
}

# 显示帮助
show_help() {
    echo "AgentX Proxy 性能测试脚本"
    echo ""
    echo "用法: $0 [选项]"
    echo ""
    echo "选项:"
    echo "  -h, --help              显示此帮助信息"
    echo "  -p, --pool-size SIZE    设置连接池大小 (默认: 5)"
    echo "  -d, --duration SEC      设置测试持续时间 (默认: 30)"
    echo "  -c, --concurrent NUM    设置并发用户数 (默认: 10)"
    echo "  -s, --stability         运行稳定性测试"
    echo ""
    echo "环境变量:"
    echo "  POOL_SIZE               连接池大小"
    echo "  TEST_DURATION           测试持续时间（秒）"
    echo "  CONCURRENT_USERS        并发用户数"
    echo ""
    echo "示例:"
    echo "  $0                      # 使用默认设置"
    echo "  $0 -p 10                # 连接池大小为 10"
    echo "  $0 -d 60 -c 20          # 测试 60 秒，20 个并发用户"
    echo "  $0 --stability          # 运行稳定性测试"
}

# 主函数
main() {
    local run_stability=false

    # 解析参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -p|--pool-size)
                POOL_SIZE="$2"
                shift 2
                ;;
            -d|--duration)
                TEST_DURATION="$2"
                shift 2
                ;;
            -c|--concurrent)
                CONCURRENT_USERS="$2"
                shift 2
                ;;
            -s|--stability)
                run_stability=true
                shift
                ;;
            *)
                error "未知参数: $1"
                show_help
                exit 1
                ;;
        esac
    done

    echo ""
    echo "========================================"
    echo "   AgentX Proxy 性能测试"
    echo "========================================"
    echo ""
    echo "配置:"
    echo "  连接池大小: $POOL_SIZE"
    echo "  测试持续时间: ${TEST_DURATION}s"
    echo "  并发用户数: $CONCURRENT_USERS"
    echo "  结果文件: $RESULTS_FILE"
    echo ""

    # 检查依赖
    if ! command -v curl &> /dev/null; then
        error "curl 未安装"
        exit 1
    fi

    if ! command -v bc &> /dev/null; then
        error "bc 未安装 (用于浮点计算)"
        exit 1
    fi

    # 注册清理陷阱
    trap cleanup EXIT

    # 启动服务
    cleanup
    start_server
    start_test_service

    # 运行测试
    if [ "$run_stability" = true ]; then
        run_stability_test $TEST_DURATION
    else
        # 启动多个客户端
        start_clients $CONCURRENT_USERS

        # 运行并发测试
        log "运行主要性能测试..."
        local num_requests=$((CONCURRENT_USERS * 3))
        run_concurrent_test $num_requests "benchmark-client-1"

        # 测试连接池
        test_connection_pool
    fi

    success "性能测试完成"
    echo ""
    echo "结果已保存到: $RESULTS_FILE"
}

# 运行主函数
main "$@"
