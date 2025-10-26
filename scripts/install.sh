#!/bin/bash
set -e

# 检测操作系统和架构
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case $OS in
  linux)
    case $ARCH in
      x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
      aarch64|arm64) TARGET="aarch64-unknown-linux-gnu" ;;
      *) echo "❌ 不支持的架构: $ARCH"; exit 1 ;;
    esac
    ;;
  darwin)
    case $ARCH in
      x86_64) TARGET="x86_64-apple-darwin" ;;
      arm64) TARGET="aarch64-apple-darwin" ;;
      *) echo "❌ 不支持的架构: $ARCH"; exit 1 ;;
    esac
    ;;
  *)
    echo "❌ 不支持的操作系统: $OS"
    exit 1
    ;;
esac

echo "检测到系统: $OS ($ARCH)"
echo "目标平台: $TARGET"
echo ""

read -p "Control port [17001]: " CONTROL_PORT
read -p "Proxy port [17002]: " PROXY_PORT
read -p "Public port [17003]: " PUBLIC_PORT
CONTROL_PORT=${CONTROL_PORT:-17001}
PROXY_PORT=${PROXY_PORT:-17002}
PUBLIC_PORT=${PUBLIC_PORT:-17003}

# R2 对象存储 URL
R2_BASE_URL="https://s3.agentx.plus"
ARCHIVE_NAME="agentx-${TARGET}.tar.gz"
DOWNLOAD_URL="${R2_BASE_URL}/builds/latest/${ARCHIVE_NAME}"

echo "正在从 R2 下载 agents..."
echo "URL: $DOWNLOAD_URL"

# 下载并解压
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

if ! curl -fL --progress-bar --max-time 300 "$DOWNLOAD_URL" -o "$ARCHIVE_NAME"; then
    echo "❌ 下载失败"
    rm -rf "$TEMP_DIR"
    exit 1
fi

echo "✅ 下载完成"

tar xzf "$ARCHIVE_NAME"

if [ ! -f "agents" ]; then
    echo "❌ 解压后未找到 agents 二进制文件"
    rm -rf "$TEMP_DIR"
    exit 1
fi

chmod +x agents
sudo mv agents /usr/local/bin/agents
cd -
rm -rf "$TEMP_DIR"

echo "✅ agents 已安装到 /usr/local/bin/agents"

sudo tee /etc/systemd/system/agents.service > /dev/null <<EOF
[Unit]
Description=agentS Server
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/agents --control-port $CONTROL_PORT --proxy-port $PROXY_PORT --public-port $PUBLIC_PORT
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable agents
sudo systemctl start agents
echo "agentS installed and started successfully"
echo "Control: $CONTROL_PORT, Proxy: $PROXY_PORT, Public: $PUBLIC_PORT"
