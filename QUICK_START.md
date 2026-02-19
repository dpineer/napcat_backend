# NapCat Backend 快速启动指南

## 🚀 一键启动

### 步骤1: 环境检查
```bash
./check.sh
```

### 步骤2: 启动服务 (开发模式)
```bash
./dev.sh
```

### 步骤3: 测试服务
```bash
./test.sh
```

## 📋 详细步骤

### 1. 首次使用
```bash
# 1. 检查环境
./check.sh

# 2. 如果需要，初始化数据库
./init_db.sh

# 3. 启动开发模式
./dev.sh
```

### 2. 日常开发
```bash
# 直接启动开发模式
./dev.sh
```

### 3. 生产部署
```bash
# 1. 检查环境
./check.sh

# 2. 初始化数据库
./init_db.sh

# 3. 启动生产模式
./run.sh
```

## 🔧 脚本说明

| 脚本 | 用途 | 使用场景 |
|-----|-----|----------|
| `check.sh` | 环境检查 | 首次使用/故障排查 |
| `init_db.sh` | 数据库初始化 | 首次部署/数据库重置 |
| `dev.sh` | 开发模式 | 日常开发/调试 |
| `run.sh` | 生产模式 | 生产环境 |
| `test.sh` | 功能测试 | 验证服务状态 |

## 📊 服务状态

服务启动后，可以通过以下方式验证：

### WebSocket服务
- 地址: `ws://localhost:8082`
- 备用地址: `ws://localhost:8082/ws`

### HTTP API
- 配置信息: `http://localhost:8082/api/config`
- 系统信息: `http://localhost:8082/api/system/info`
- 知识库: `http://localhost:8082/api/knowledge`
- 提示词: `http://localhost:8082/api/prompts`

### 预期响应
访问 `http://localhost:8082/api/config` 应该返回：
```json
{
  "status": "success",
  "data": {
    "llm_base_url": "https://api.deepseek.com",
    "llm_model": "deepseek-chat",
    "prompt_stats": { ... },
    "system_info": { ... }
  }
}
```

## ⚡ 快速测试

### 测试知识库功能
```bash
# 添加知识
curl -X POST http://localhost:8082/api/knowledge \
  -H "Content-Type: application/json" \
  -d '{"content": "Rust是一种系统编程语言"}'

# 搜索知识
curl -X POST http://localhost:8082/api/knowledge \
  -H "Content-Type: application/json" \
  -d '{"query": "Rust"}'
```

### 测试提示词功能
```bash
# 获取提示词信息
curl http://localhost:8082/api/prompts

# 获取提示词统计
curl http://localhost:8082/api/prompts/stats
```

## 🛠️ 故障排查

### 服务无法启动
1. **检查端口占用**:
   ```bash
   ./check.sh
   ```

2. **查看详细日志**:
   ```bash
   RUST_LOG=debug ./dev.sh
   ```

3. **检查配置文件**:
   ```bash
   cat config.toml
   cat ~/.config/napcat-backend/secrets.env
   ```

### API无响应
1. **检查服务状态**:
   ```bash
   curl http://localhost:8082/api/config
   ```

2. **检查网络连接**:
   ```bash
   netstat -tulnp | grep 8082
   ```

### 数据库错误
1. **检查数据库连接**:
   ```bash
   ./check.sh
   ```

2. **重新初始化数据库**:
   ```bash
   ./init_db.sh
   ```

## 📈 性能监控

### 基本监控
```bash
# 查看服务状态
./test.sh

# 查看系统资源
top -p $(pgrep napcat_backend)
```

### 日志监控
```bash
# 实时查看日志
tail -f napcat.log

# 查看错误日志
grep ERROR napcat.log
```

## 🔒 安全建议

1. **保护API密钥**: 不要泄露 `LLM_API_KEY`
2. **数据库安全**: 使用强密码，限制访问
3. **网络安全**: 配置防火墙规则
4. **日志安全**: 避免记录敏感信息

## 📚 更多资源

- [完整文档](RUN_SCRIPTS.md) - 详细的脚本使用指南
- [API文档](API_DOCUMENTATION.md) - API接口说明
- [项目文档](完整项目文档.md) - 项目架构说明

## 🆘 获取帮助

如果遇到问题：

1. **查看日志**: 服务启动时的输出信息
2. **运行诊断**: `./check.sh`
3. **检查配置**: 确认环境变量设置正确
4. **网络诊断**: 确认端口未被占用

## ✅ 成功指标

服务正常运行时，你应该看到：

1. **启动日志**:
   ```
   ✅ WebSocket server listening on 0.0.0.0:8082
   ✅ Management API available at http://0.0.0.0:8082/api/*
   ```

2. **API响应**: 访问 `http://localhost:8082/api/config` 返回JSON数据

3. **端口监听**: `netstat -tulnp | grep 8082` 显示服务正在监听

## 🎯 下一步

服务启动后，你可以：

1. **连接WebSocket客户端**到 `ws://localhost:8082`
2. **使用管理API**进行配置和监控
3. **添加知识库内容**通过API
4. **测试AI对话功能**

---

**💡 提示**: 建议首次使用时先运行 `./check.sh` 确保环境配置正确！