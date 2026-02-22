#!/bin/bash

# 统一启动脚本 (NapCat Backend + Open-LLM-VTuber)
# 用于启动整合了napcat_backend和Open-LLM-VTuber功能的服务

set -e  # 遇到错误时退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印带颜色的信息
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查是否在项目根目录
if [ ! -f "Cargo.toml" ]; then
    print_error "未在项目根目录中找到 Cargo.toml 文件"
    print_error "请确保在 napcat_backend 项目目录中运行此脚本"
    exit 1
fi

# 检查环境变量
print_info "检查环境变量配置..."

if [ ! -f "$HOME/.config/napcat-backend/secrets.env" ]; then
    print_error "未找到环境变量配置文件: $HOME/.config/napcat-backend/secrets.env"
    print_error "请先创建配置文件并设置必要的环境变量"
    print_info "示例配置文件内容:"
    echo "
DATABASE_URL=postgresql://username:password@localhost/database_name
LLM_API_KEY=your_api_key_here
LLM_BASE_URL=https://api.openai.com/v1
LLM_MODEL=gpt-4
    "
    exit 1
fi

# 加载环境变量
print_info "加载环境变量..."
source "$HOME/.config/napcat-backend/secrets.env"

# 检查必要的环境变量
required_vars=("DATABASE_URL" "LLM_API_KEY" "LLM_BASE_URL" "LLM_MODEL")
for var in "${required_vars[@]}"; do
    if [ -z "${!var}" ]; then
        print_error "环境变量 $var 未设置"
        exit 1
    fi
done

print_success "环境变量检查通过"

# 检查 Rust 工具链
print_info "检查 Rust 工具链..."
if ! command -v cargo &> /dev/null; then
    print_error "未找到 cargo 命令，请先安装 Rust 工具链"
    print_info "安装 Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

if ! command -v rustc &> /dev/null; then
    print_error "未找到 rustc 命令，请先安装 Rust 工具链"
    exit 1
fi

print_success "Rust 工具链检查通过"

# 编译项目
print_info "编译项目..."
cargo build --release --bin unified_launcher

if [ $? -eq 0 ]; then
    print_success "编译成功"
else
    print_error "编译失败"
    exit 1
fi

# 检查数据库连接
print_info "检查数据库连接..."
if ! cargo run --release --bin napcat_backend -- check-db; then
    print_warning "数据库连接检查失败，但继续启动服务"
fi

# 启动统一服务
print_info "🚀 启动统一服务 (NapCat Backend + Open-LLM-VTuber)..."
print_info "按 Ctrl+C 停止服务"

# 运行统一启动器
exec cargo run --release --bin unified_launcher