#!/bin/bash

# NapCat Backend 环境检查脚本

echo "=== NapCat Backend 环境诊断 ==="
echo "当前时间: $(date)"
echo "工作目录: $(pwd)"
echo ""

# 检查系统信息
echo "=== 系统信息 ==="
echo "操作系统: $(uname -a)"
echo "CPU架构: $(uname -m)"
echo "内存信息: $(free -h | grep Mem | awk '{print $2}')"
echo ""

# 检查Rust环境
echo "=== Rust环境检查 ==="
if command -v rustc &> /dev/null; then
    echo "✅ Rust已安装"
    echo "版本: $(rustc --version)"
    echo "安装路径: $(which rustc)"
else
    echo "❌ Rust未安装"
    echo "请访问 https://rustup.rs/ 安装Rust"
fi

if command -v cargo &> /dev/null; then
    echo "✅ Cargo已安装"
    echo "版本: $(cargo --version)"
else
    echo "❌ Cargo未安装"
fi

if [ -f "$HOME/.cargo/env" ]; then
    echo "✅ Cargo环境脚本存在"
else
    echo "⚠️  Cargo环境脚本不存在"
fi
echo ""

# 检查PostgreSQL
echo "=== PostgreSQL检查 ==="
if command -v psql &> /dev/null; then
    echo "✅ PostgreSQL客户端已安装"
    echo "版本: $(psql --version)"
    
    # 检查数据库连接
    if [ -f "$HOME/.config/napcat-backend/secrets.env" ]; then
        source "$HOME/.config/napcat-backend/secrets.env"
        echo "正在测试数据库连接..."
        if psql "$DATABASE_URL" -c "SELECT 1;" &> /dev/null; then
            echo "✅ 数据库连接正常"
            echo "数据库: $(psql "$DATABASE_URL" -c "SELECT current_database();" -t -A)"
        else
            echo "❌ 数据库连接失败"
            echo "请检查:"
            echo "  1. PostgreSQL服务是否运行: sudo systemctl status postgresql"
            echo "  2. 数据库是否存在: createdb napcat_backend"
            echo "  3. 连接参数是否正确"
        fi
    else
        echo "⚠️  未找到环境配置文件，跳过数据库连接测试"
    fi
else
    echo "❌ PostgreSQL客户端未安装"
    echo "请安装: sudo apt-get install postgresql-client"
fi
echo ""

# 检查项目文件
echo "=== 项目文件检查 ==="
required_files=(
    "Cargo.toml"
    "src/main.rs"
    "src/models.rs"
    "src/db.rs"
    "src/knowledge_base.rs"
    "src/context_manager.rs"
    "src/prompts.rs"
    "src/enhanced_prompts.rs"
    "src/config.rs"
    "migrations/001_initial_schema.sql"
    "config.example.toml"
)

for file in "${required_files[@]}"; do
    if [ -f "$file" ]; then
        echo "✅ $file"
    else
        echo "❌ 缺少文件: $file"
    fi
done
echo ""

# 检查配置文件
echo "=== 配置文件检查 ==="
if [ -f "$HOME/.config/napcat-backend/secrets.env" ]; then
    echo "✅ 环境配置文件存在: $HOME/.config/napcat-backend/secrets.env"
    source "$HOME/.config/napcat-backend/secrets.env"
    
    # 检查必要的环境变量
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
else
    echo "❌ 未找到环境配置文件: $HOME/.config/napcat-backend/secrets.env"
fi

if [ -f "config.toml" ]; then
    echo "✅ 主配置文件存在: config.toml"
else
    echo "⚠️  主配置文件不存在，将使用默认配置"
fi
echo ""

# 检查依赖
echo "=== 依赖检查 ==="
echo "正在检查Cargo依赖..."
if cargo check &> /dev/null; then
    echo "✅ Cargo依赖检查通过"
else
    echo "❌ Cargo依赖检查失败"
    echo "运行 'cargo check' 查看详细错误"
fi
echo ""

# 检查端口
echo "=== 端口检查 ==="
if command -v netstat &> /dev/null; then
    if netstat -tuln | grep -q ":8082"; then
        echo "⚠️  端口8082已被占用"
        echo "占用进程:"
        netstat -tulnp | grep ":8082"
    else
        echo "✅ 端口8082可用"
    fi
elif command -v ss &> /dev/null; then
    if ss -tuln | grep -q ":8082"; then
        echo "⚠️  端口8082已被占用"
        echo "占用进程:"
        ss -tulnp | grep ":8082"
    else
        echo "✅ 端口8082可用"
    fi
else
    echo "⚠️  未找到端口检查工具"
fi
echo ""

# 总结
echo "=== 诊断总结 ==="
echo "如果所有检查项都显示✅，则可以尝试运行:"
echo "  ./dev.sh  # 开发模式"
echo "  ./run.sh  # 生产模式"
echo ""
echo "如果有❌错误，请先解决相应问题"
echo "诊断完成时间: $(date)"