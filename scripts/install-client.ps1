$ErrorActionPreference = "Stop"

# 检测架构
$ARCH = $env:PROCESSOR_ARCHITECTURE
switch ($ARCH) {
    "AMD64" { $TARGET = "x86_64-pc-windows-msvc" }
    "ARM64" { $TARGET = "aarch64-pc-windows-msvc" }
    default {
        Write-Host "❌ 不支持的架构: $ARCH" -ForegroundColor Red
        exit 1
    }
}

Write-Host "检测到系统: Windows ($ARCH)"
Write-Host "目标平台: $TARGET"
Write-Host ""

# R2 对象存储 URL
$R2_BASE_URL = "https://s3.agentx.plus"
$ARCHIVE_NAME = "arp-${TARGET}.zip"
$DOWNLOAD_URL = "${R2_BASE_URL}/builds/latest/${ARCHIVE_NAME}"

Write-Host "正在从 R2 下载 arpc..."
Write-Host "URL: $DOWNLOAD_URL"

# 创建临时目录
$TEMP_DIR = New-Item -ItemType Directory -Path (Join-Path $env:TEMP ([System.IO.Path]::GetRandomFileName()))

try {
    # 下载文件
    $ARCHIVE_PATH = Join-Path $TEMP_DIR $ARCHIVE_NAME
    Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile $ARCHIVE_PATH -TimeoutSec 300
    Write-Host "✅ 下载完成" -ForegroundColor Green

    # 解压
    Expand-Archive -Path $ARCHIVE_PATH -DestinationPath $TEMP_DIR -Force

    $BINARY_PATH = Join-Path $TEMP_DIR "arpc.exe"
    if (-not (Test-Path $BINARY_PATH)) {
        Write-Host "❌ 解压后未找到 arpc.exe 二进制文件" -ForegroundColor Red
        exit 1
    }

    # 安装到用户目录
    $INSTALL_DIR = Join-Path $env:USERPROFILE "bin"
    $INSTALL_PATH = Join-Path $INSTALL_DIR "arpc.exe"

    if (-not (Test-Path $INSTALL_DIR)) {
        New-Item -ItemType Directory -Path $INSTALL_DIR | Out-Null
    }

    Copy-Item -Path $BINARY_PATH -Destination $INSTALL_PATH -Force
    Write-Host "✅ arpc 已安装到 $INSTALL_PATH" -ForegroundColor Green

    # 检查并添加到 PATH
    $USER_PATH = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($USER_PATH -notlike "*$INSTALL_DIR*") {
        Write-Host ""
        Write-Host "正在将 $INSTALL_DIR 添加到 PATH..." -ForegroundColor Yellow
        [Environment]::SetEnvironmentVariable("Path", "$USER_PATH;$INSTALL_DIR", "User")
        Write-Host "✅ PATH 已更新（需要重启终端生效）" -ForegroundColor Green
    }

    Write-Host ""
    Write-Host "安装完成！重启终端后运行 'arpc --help' 查看使用说明" -ForegroundColor Green
}
finally {
    # 清理临时文件
    Remove-Item -Path $TEMP_DIR -Recurse -Force -ErrorAction SilentlyContinue
}
