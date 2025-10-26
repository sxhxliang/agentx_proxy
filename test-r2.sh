#!/bin/bash

# R2 验证脚本
# 使用方法: ./test-r2.sh


set -e

echo "=== Cloudflare R2 环境变量验证 ==="
echo ""

# 检查必需的环境变量
if [ -z "$R2_ACCESS_KEY_ID" ]; then
    echo "❌ 错误: R2_ACCESS_KEY_ID 未设置"
    exit 1
fi

if [ -z "$R2_SECRET_ACCESS_KEY" ]; then
    echo "❌ 错误: R2_SECRET_ACCESS_KEY 未设置"
    exit 1
fi

if [ -z "$R2_ACCOUNT_ID" ]; then
    echo "❌ 错误: R2_ACCOUNT_ID 未设置"
    exit 1
fi

if [ -z "$R2_BUCKET" ]; then
    echo "❌ 错误: R2_BUCKET 未设置"
    exit 1
fi

echo "✅ 所有环境变量已设置"
echo ""

# 配置 AWS CLI
export AWS_ACCESS_KEY_ID=$R2_ACCESS_KEY_ID
export AWS_SECRET_ACCESS_KEY=$R2_SECRET_ACCESS_KEY
export AWS_DEFAULT_REGION=auto
R2_ENDPOINT="https://${R2_ACCOUNT_ID}.r2.cloudflarestorage.com"

echo "R2 配置:"
echo "  Account ID: $R2_ACCOUNT_ID"
echo "  Bucket: $R2_BUCKET"
echo "  Endpoint: $R2_ENDPOINT"
echo ""

# 测试连接
echo "测试 1: 列出存储桶..."
if aws s3 ls --endpoint-url "$R2_ENDPOINT" 2>/dev/null; then
    echo "✅ 成功列出存储桶"
else
    echo "❌ 无法列出存储桶 - 请检查凭证"
    exit 1
fi
echo ""

# 测试访问指定存储桶
echo "测试 2: 访问存储桶 $R2_BUCKET..."
if aws s3 ls "s3://${R2_BUCKET}/" --endpoint-url "$R2_ENDPOINT" 2>/dev/null; then
    echo "✅ 成功访问存储桶"
else
    echo "❌ 无法访问存储桶 - 请检查存储桶名称和权限"
    exit 1
fi
echo ""

# 测试上传
echo "测试 3: 上传测试文件..."
TEST_FILE=$(mktemp)
echo "R2 test at $(date)" > "$TEST_FILE"
TEST_KEY="test/r2-test-$(date +%s).txt"

if aws s3 cp "$TEST_FILE" "s3://${R2_BUCKET}/${TEST_KEY}" --endpoint-url "$R2_ENDPOINT" 2>/dev/null; then
    echo "✅ 成功上传测试文件"

    # 测试删除
    echo "测试 4: 删除测试文件..."
    if aws s3 rm "s3://${R2_BUCKET}/${TEST_KEY}" --endpoint-url "$R2_ENDPOINT" 2>/dev/null; then
        echo "✅ 成功删除测试文件"
    else
        echo "⚠️  上传成功但删除失败"
    fi
else
    echo "❌ 无法上传文件 - 请检查写入权限"
    exit 1
fi

rm -f "$TEST_FILE"
echo ""
echo "🎉 所有测试通过！R2 配置有效。"
