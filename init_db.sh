#!/bin/bash

# NapCat Backend 数据库初始化脚本

echo "=== NapCat Backend 数据库初始化 ==="
echo "当前时间: $(date)"
echo ""

# 检查环境文件
if [ -f "$HOME/.config/napcat-backend/secrets.env" ]; then
    echo "✅ 找到环境配置文件: $HOME/.config/napcat-backend/secrets.env"
    source "$HOME/.config/napcat-backend/secrets.env"
else
    echo "❌ 未找到环境配置文件: $HOME/.config/napcat-backend/secrets.env"
    echo "请先创建配置文件"
    exit 1
fi

# 检查PostgreSQL客户端
if ! command -v psql &> /dev/null; then
    echo "❌ PostgreSQL客户端未安装"
    echo "请安装: sudo apt-get install postgresql-client"
    exit 1
fi

echo "=== 数据库连接测试 ==="
if psql "$DATABASE_URL" -c "SELECT 1;" &> /dev/null; then
    echo "✅ 数据库连接正常"
    echo "数据库: $(psql "$DATABASE_URL" -c "SELECT current_database();" -t -A)"
else
    echo "❌ 数据库连接失败"
    echo "请检查:"
    echo "  1. PostgreSQL服务是否运行: sudo systemctl status postgresql"
    echo "  2. 数据库是否存在: createdb napcat_backend"
    echo "  3. 连接参数是否正确"
    exit 1
fi

echo ""
echo "=== 检查数据库扩展 ==="
# 检查pgvector扩展
if psql "$DATABASE_URL" -c "SELECT extname FROM pg_extension WHERE extname = 'vector';" | grep -q "vector"; then
    echo "✅ pgvector扩展已安装"
else
    echo "⚠️  pgvector扩展未安装"
    echo "正在尝试安装pgvector扩展..."
    if psql "$DATABASE_URL" -c "CREATE EXTENSION IF NOT EXISTS vector;" &> /dev/null; then
        echo "✅ pgvector扩展安装成功"
    else
        echo "❌ pgvector扩展安装失败"
        echo "请手动安装pgvector扩展:"
        echo "  sudo apt-get install postgresql-pgvector"
        echo "  或在数据库中执行: CREATE EXTENSION vector;"
    fi
fi

echo ""
echo "=== 执行数据库迁移 ==="
if [ -f "migrations/001_initial_schema.sql" ]; then
    echo "正在执行数据库迁移..."
    if psql "$DATABASE_URL" -f "migrations/001_initial_schema.sql"; then
        echo "✅ 数据库迁移成功"
    else
        echo "❌ 数据库迁移失败"
        exit 1
    fi
else
    echo "❌ 未找到迁移文件: migrations/001_initial_schema.sql"
    exit 1
fi

echo ""
echo "=== 验证数据库表 ==="
tables=("conversations" "knowledge_base" "user_contexts" "group_contexts" "system_configs")

for table in "${tables[@]}"; do
    if psql "$DATABASE_URL" -c "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = '$table');" | grep -q "t"; then
        echo "✅ 表 $table 存在"
    else
        echo "❌ 表 $table 不存在"
    fi
done

echo ""
echo "=== 数据库统计 ==="
echo "对话记录数: $(psql "$DATABASE_URL" -c "SELECT COUNT(*) FROM conversations;" -t -A)"
echo "知识库条目数: $(psql "$DATABASE_URL" -c "SELECT COUNT(*) FROM knowledge_base;" -t -A)"
echo "用户上下文数: $(psql "$DATABASE_URL" -c "SELECT COUNT(*) FROM user_contexts;" -t -A)"
echo "群组上下文数: $(psql "$DATABASE_URL" -c "SELECT COUNT(*) FROM group_contexts;" -t -A)"

echo ""
echo "=== 数据库初始化完成 ==="
echo "✅ 数据库已准备就绪"
echo "现在可以运行:"
echo "  ./dev.sh  # 开发模式"
echo "  ./run.sh  # 生产模式"
echo ""
echo "完成时间: $(date)"