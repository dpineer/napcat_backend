#!/bin/bash

# NapCat Backend 开发模式运行脚本

echo "=== NapCat Backend 开发模式启动脚本 ==="
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

# 设置开发环境变量
export RUST_LOG=debug
export RUST_BACKTRACE=1

echo ""
echo "=== 开发环境设置 ==="
echo "日志级别: $RUST_LOG"
echo "错误回溯: $RUST_BACKTRACE"

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

# 运行开发模式
echo ""
echo "=== 启动NapCat Backend (开发模式) ==="
echo "正在启动服务..."
echo "服务将在端口8082上监听"
echo "管理API地址: http://localhost:8082/api/"
echo "日志级别: DEBUG"
echo "按 Ctrl+C 停止服务"
echo ""

# 使用cargo run运行开发模式
cargo run