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
    BINARY="arpc"
    ;;
  darwin)
    case $ARCH in
      x86_64) TARGET="x86_64-apple-darwin" ;;
      arm64) TARGET="aarch64-apple-darwin" ;;
      *) echo "❌ 不支持的架构: $ARCH"; exit 1 ;;
    esac
    BINARY="arpc"
    ;;
  mingw*|msys*|cygwin*)
    TARGET="x86_64-pc-windows-msvc"
    BINARY="arpc.exe"
    ;;
  *)
    echo "❌ 不支持的操作系统: $OS"
    exit 1
    ;;
esac

echo "检测到系统: $OS ($ARCH)"
echo "目标平台: $TARGET"
echo ""

# R2 对象存储 URL
R2_BASE_URL="https://s3.agentx.plus"

if [[ $TARGET == *"windows"* ]]; then
  ARCHIVE_NAME="agentx-${TARGET}.zip"
else
  ARCHIVE_NAME="agentx-${TARGET}.tar.gz"
fi

DOWNLOAD_URL="${R2_BASE_URL}/builds/latest/${ARCHIVE_NAME}"

echo "正在从 R2 下载 arpc..."
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

# 解压
if [[ $ARCHIVE_NAME == *.zip ]]; then
  unzip -q "$ARCHIVE_NAME"
else
  tar xzf "$ARCHIVE_NAME"
fi

if [ ! -f "$BINARY" ]; then
    echo "❌ 解压后未找到 $BINARY 二进制文件"
    rm -rf "$TEMP_DIR"
    exit 1
fi

chmod +x "$BINARY"

# 安装到系统路径
if [[ "$OS" == "darwin" ]] || [[ "$OS" == "linux" ]]; then
  INSTALL_PATH="/usr/local/bin/arpc"
  sudo mv "$BINARY" "$INSTALL_PATH"
  echo "✅ arpc 已安装到 $INSTALL_PATH"
else
  # Windows
  INSTALL_PATH="$HOME/bin/arpc.exe"
  mkdir -p "$HOME/bin"
  mv "$BINARY" "$INSTALL_PATH"
  echo "✅ arpc 已安装到 $INSTALL_PATH"
  echo "请确保 $HOME/bin 在 PATH 环境变量中"
fi

cd -
rm -rf "$TEMP_DIR"

echo ""
echo "安装完成！运行 'arpc --help' 查看使用说明"
