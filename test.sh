#!/bin/bash

# NapCat Backend 测试脚本

echo "=== NapCat Backend 功能测试 ==="
echo "当前时间: $(date)"
echo ""

# 检查环境文件
if [ -f "$HOME/.config/napcat-backend/secrets.env" ]; then
    echo "✅ 找到环境配置文件"
    source "$HOME/.config/napcat-backend/secrets.env"
else
    echo "❌ 未找到环境配置文件"
    exit 1
fi

# 测试编译
echo "=== 编译测试 ==="
echo "正在测试编译..."
if cargo build --release &> /dev/null; then
    echo "✅ 编译成功"
else
    echo "❌ 编译失败"
    exit 1
fi

# 测试API端点
echo ""
echo "=== API端点测试 ==="
echo "测试管理API端点..."

# 测试配置端点
echo "正在测试配置端点..."
if curl -s -o /dev/null -w "%{http_code}" http://localhost:8082/api/config | grep -q "200\|503"; then
    echo "✅ 配置端点响应正常"
else
    echo "⚠️  配置端点未响应 (服务可能未启动)"
fi

# 测试知识库端点
echo "正在测试知识库端点..."
if curl -s -o /dev/null -w "%{http_code}" http://localhost:8082/api/knowledge | grep -q "200\|503"; then
    echo "✅ 知识库端点响应正常"
else
    echo "⚠️  知识库端点未响应 (服务可能未启动)"
fi

# 测试提示词端点
echo "正在测试提示词端点..."
if curl -s -o /dev/null -w "%{http_code}" http://localhost:8082/api/prompts | grep -q "200\|503"; then
    echo "✅ 提示词端点响应正常"
else
    echo "⚠️  提示词端点未响应 (服务可能未启动)"
fi

# 测试系统信息端点
echo "正在测试系统信息端点..."
if curl -s -o /dev/null -w "%{http_code}" http://localhost:8082/api/system/info | grep -q "200\|503"; then
    echo "✅ 系统信息端点响应正常"
else
    echo "⚠️  系统信息端点未响应 (服务可能未启动)"
fi

echo ""
echo "=== WebSocket连接测试 ==="
echo "正在测试WebSocket端口..."
if command -v nc &> /dev/null; then
    if timeout 2 nc -z localhost 8082; then
        echo "✅ WebSocket端口8082可连接"
    else
        echo "⚠️  WebSocket端口8082不可连接 (服务可能未启动)"
    fi
else
    echo "⚠️  未安装nc命令，跳过端口测试"
fi

echo ""
echo "=== 环境变量测试 ==="
echo "正在检查必要的环境变量..."

required_vars=(
    "LLM_API_KEY"
    "LLM_BASE_URL"
    "LLM_MODEL"
    "DATABASE_URL"
    "NAPCAT_TOKEN"
)

for var in "${required_vars[@]}"; do
    if [ -n "${!var}" ]; then
        echo "✅ $var 已设置"
    else
        echo "❌ $var 未设置"
    fi
done

echo ""
echo "=== 数据库连接测试 ==="
echo "正在测试数据库连接..."
if command -v psql &> /dev/null; then
    if psql "$DATABASE_URL" -c "SELECT 1;" &> /dev/null; then
        echo "✅ 数据库连接正常"
    else
        echo "❌ 数据库连接失败"
    fi
else
    echo "⚠️  未安装PostgreSQL客户端，跳过数据库测试"
fi

echo ""
echo "=== 测试总结 ==="
echo "基本功能测试完成"
echo ""
echo "如果服务未启动，可以运行:"
echo "  ./dev.sh  # 开发模式 (带调试日志)"
echo "  ./run.sh  # 生产模式"
echo ""
echo "如果数据库未初始化，可以运行:"
echo "  ./init_db.sh  # 初始化数据库"
echo ""
echo "测试完成时间: $(date)"