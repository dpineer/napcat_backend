#!/bin/bash

# NapCat Backend 运行脚本

echo "=== NapCat Backend 启动脚本 ==="
echo "当前时间: $(date)"
echo "工作目录: $(pwd)"

# 检查环境文件
if [ -f "$HOME/.config/napcat-backend/secrets.env" ]; then
    echo "✅ 找到环境配置文件: $HOME/.config/napcat-backend/secrets.env"
    source "$HOME/.config/napcat-backend/secrets.env"
else
    echo "❌ 未找到环境配置文件: $HOME/.config/napcat-backend/secrets.env"
    echo "请先创建配置文件"
    exit 1
fi

# 检查配置文件
if [ -f "config.toml" ]; then
    echo "✅ 找到配置文件: config.toml"
else
    echo "⚠️  未找到配置文件 config.toml，使用示例配置"
    if [ -f "config.example.toml" ]; then
        cp config.example.toml config.toml
        echo "✅ 已复制示例配置文件"
    else
        echo "❌ 未找到示例配置文件"
        exit 1
    fi
fi

# 检查数据库连接
echo ""
echo "=== 检查数据库连接 ==="
if command -v psql &> /dev/null; then
    echo "正在测试数据库连接..."
    if psql "$DATABASE_URL" -c "SELECT 1;" &> /dev/null; then
        echo "✅ 数据库连接正常"
    else
        echo "❌ 数据库连接失败"
        echo "请确保PostgreSQL正在运行且数据库已创建"
        echo "数据库URL: $DATABASE_URL"
        exit 1
    fi
else
    echo "⚠️  未安装psql命令，跳过数据库连接测试"
fi

# 检查Rust环境
echo ""
echo "=== 检查Rust环境 ==="
if command -v cargo &> /dev/null; then
    echo "✅ Rust环境正常"
    echo "Cargo版本: $(cargo --version)"
else
    echo "❌ 未安装Rust/Cargo"
    echo "请先安装Rust: https://rustup.rs/"
    exit 1
fi

# 构建项目
echo ""
echo "=== 构建项目 ==="
echo "开始构建项目..."
if cargo build --release; then
    echo "✅ 项目构建成功"
else
    echo "❌ 项目构建失败"
    exit 1
fi

# 运行项目
echo ""
echo "=== 启动NapCat Backend ==="
echo "正在启动服务..."
echo "服务将在端口8082上监听"
echo "管理API地址: http://localhost:8082/api/"
echo "按 Ctrl+C 停止服务"
echo ""

# 设置日志级别
export RUST_LOG=${RUST_LOG:-info}

# 运行程序
./target/release/napcat_backend