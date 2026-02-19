# NapCat Backend 运行脚本使用指南

## 脚本列表

### 1. check.sh - 环境检查脚本
**用途**: 检查运行环境是否满足要求
**使用方法**:
```bash
./check.sh
```

**检查项目**:
- Rust/Cargo环境
- PostgreSQL客户端
- 项目文件完整性
- 配置文件存在性
- 环境变量设置
- 依赖项检查
- 端口可用性

### 2. init_db.sh - 数据库初始化脚本
**用途**: 初始化数据库和表结构
**使用方法**:
```bash
./init_db.sh
```

**功能**:
- 测试数据库连接
- 安装pgvector扩展
- 执行数据库迁移
- 验证表结构
- 显示数据库统计

### 3. dev.sh - 开发模式运行脚本
**用途**: 以开发模式运行服务
**使用方法**:
```bash
./dev.sh
```

**特点**:
- 使用`cargo run`运行
- 日志级别: DEBUG
- 错误回溯: 启用
- 实时编译
- 适合开发和调试

### 4. run.sh - 生产模式运行脚本
**用途**: 以生产模式运行服务
**使用方法**:
```bash
./run.sh
```

**特点**:
- 使用编译后的二进制文件
- 日志级别: INFO (可配置)
- 性能优化
- 适合生产环境

### 5. test.sh - 功能测试脚本
**用途**: 测试基本功能
**使用方法**:
```bash
./test.sh
```

**测试项目**:
- 编译测试
- API端点测试
- WebSocket连接测试
- 环境变量检查
- 数据库连接测试

## 使用流程

### 首次使用
1. **环境检查**:
   ```bash
   ./check.sh
   ```

2. **数据库初始化** (如果需要):
   ```bash
   ./init_db.sh
   ```

3. **开发模式运行**:
   ```bash
   ./dev.sh
   ```

### 日常开发
1. **开发模式**:
   ```bash
   ./dev.sh
   ```

2. **功能测试**:
   ```bash
   ./test.sh
   ```

### 生产部署
1. **环境检查**:
   ```bash
   ./check.sh
   ```

2. **数据库初始化**:
   ```bash
   ./init_db.sh
   ```

3. **生产模式运行**:
   ```bash
   ./run.sh
   ```

## 环境要求

### 必需软件
- **Rust**: 1.70+ (建议最新稳定版)
- **PostgreSQL**: 12+ (需要pgvector扩展)
- **PostgreSQL客户端**: psql命令

### 配置文件
- **环境配置**: `~/.config/napcat-backend/secrets.env`
- **应用配置**: `config.toml` (自动生成)

### 环境变量
```bash
# LLM API配置
LLM_API_KEY=your_api_key
LLM_BASE_URL=https://api.deepseek.com
LLM_MODEL=deepseek-chat

# NapCat配置
NAPCAT_WS_URL=ws://localhost:8082
NAPCAT_TOKEN=your_token

# 数据库配置
DATABASE_URL=postgresql://user:password@localhost:5432/napcat_backend

# 日志级别
RUST_LOG=info
```

## 常见问题

### 1. PostgreSQL连接失败
**问题**: `psql: error: connection to server failed`
**解决**:
```bash
# 检查PostgreSQL服务状态
sudo systemctl status postgresql

# 启动PostgreSQL服务
sudo systemctl start postgresql

# 创建数据库
createdb napcat_backend
```

### 2. pgvector扩展未安装
**问题**: `ERROR: could not open extension control file`
**解决**:
```bash
# Ubuntu/Debian
sudo apt-get install postgresql-pgvector

# 然后在数据库中执行
psql -d napcat_backend -c "CREATE EXTENSION vector;"
```

### 3. 端口被占用
**问题**: `Address already in use`
**解决**:
```bash
# 查看占用端口的进程
sudo netstat -tulnp | grep :8082

# 终止占用进程
sudo kill -9 <PID>
```

### 4. Rust编译错误
**问题**: `error: could not compile`
**解决**:
```bash
# 清理构建缓存
cargo clean

# 重新编译
cargo build
```

## 服务端口

- **WebSocket服务**: `localhost:8082`
- **管理API**: `http://localhost:8082/api/`

### API端点
- `GET /api/config` - 获取配置
- `GET /api/knowledge` - 搜索知识库
- `POST /api/knowledge` - 添加知识
- `GET /api/prompts` - 获取提示词信息
- `GET /api/prompts/stats` - 获取提示词统计
- `GET /api/system/info` - 获取系统信息

## 日志查看

### 开发模式
日志会直接输出到终端，包含DEBUG级别信息。

### 生产模式
建议配置日志文件:
```bash
# 在secrets.env中添加
RUST_LOG=info

# 运行并重定向日志
./run.sh > napcat.log 2>&1
```

## 性能优化

### 生产环境建议
1. 使用release模式编译
2. 配置适当的日志级别
3. 使用进程管理器(如systemd)
4. 配置数据库连接池
5. 启用数据库索引

### 系统调优
```bash
# 增加文件描述符限制
ulimit -n 65536

# 优化数据库连接
# 在postgresql.conf中调整
max_connections = 200
shared_buffers = 256MB
```

## 安全建议

1. **API密钥**: 妥善保管LLM API密钥
2. **数据库**: 使用强密码，限制访问权限
3. **网络**: 配置防火墙，限制访问IP
4. **日志**: 避免在日志中记录敏感信息
5. **更新**: 定期更新依赖包

## 故障排查

### 服务无法启动
1. 检查端口占用: `./check.sh`
2. 查看错误日志: `RUST_LOG=debug ./dev.sh`
3. 检查配置文件: `cat config.toml`
4. 验证数据库连接: `psql $DATABASE_URL`

### API无响应
1. 检查服务状态: `curl http://localhost:8082/api/config`
2. 查看网络连接: `netstat -tulnp | grep 8082`
3. 检查防火墙设置

### 数据库错误
1. 检查数据库连接: `./check.sh`
2. 验证表结构: `./init_db.sh`
3. 查看数据库日志

## 更新和维护

### 代码更新
```bash
# 拉取最新代码
git pull

# 重新编译
cargo build --release

# 重启服务
./run.sh
```

### 数据库迁移
```bash
# 备份数据库
pg_dump $DATABASE_URL > backup.sql

# 执行新迁移
psql $DATABASE_URL -f migrations/new_migration.sql
```

## 支持

如果遇到问题:
1. 查看日志文件
2. 运行诊断脚本: `./check.sh`
3. 检查GitHub Issues
4. 联系技术支持