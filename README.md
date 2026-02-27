# NapCat AI助手后端

一个基于Rust构建的智能QQ机器人后端，集成大语言模型(LLM)、知识库(RAG)、向量数据库和智能提示词管理系统。

该项目的前端管理工具使用Flutter编写,开源地址
https://github.com/dpineer/napcat_admin


## 🌟 核心特性

- **🤖 智能对话**: 基于大语言模型的自然语言理解和生成
- **📚 知识库系统**: 支持本地知识存储和向量检索(RAG)
- **🎯 智能提示词**: 多场景提示词模板和动态选择系统
- **💬 QQ集成**: 通过NapCat协议与QQ群聊无缝对接
- **🔄 上下文管理**: 对话历史跟踪和智能上下文提取
- **🔍 数据库查询**: 通过QQ指令实时检查数据库内容
- **⚡ 高性能**: 基于Rust和Tokio异步运行时
- **🔧 可配置**: 灵活的配置系统和API管理接口

## 🏗️ 系统架构

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   QQ客户端      │    │   NapCat协议    │    │   后端服务      │
│                 │◄──►│                 │◄──►│                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                        │
                       ┌─────────────────┐             │
                       │   向量数据库    │◄────────────┤
                       │   (PostgreSQL)  │             │
                       └─────────────────┘             │
                                                        │
                       ┌─────────────────┐             │
                       │   大语言模型    │◄────────────┘
                       │   (OpenAI API)  │
                       └─────────────────┘
```

## 🚀 快速开始

### 前置要求

- **Rust** (1.70+)
- **PostgreSQL** (支持pgvector扩展)
- **NapCat** (QQ机器人框架)
- **大语言模型API** (OpenAI兼容API)

### 1. 克隆项目

```bash
git clone https://github.com/dpineer/napcat_backend.git
cd napcat_backend
```

### 2. 环境配置

复制环境配置示例文件：

```bash
cp config.example.toml config.toml
cp .env.example .env
```

编辑 `.env` 文件，配置必要的环境变量：

```env
# 数据库连接
DATABASE_URL=postgres://username:password@localhost:5432/napcat_db

# LLM API配置
LLM_API_KEY=your_openai_api_key
LLM_BASE_URL=https://api.openai.com/v1
LLM_MODEL=gpt-3.5-turbo

# NapCat配置
NAPCAT_TOKEN=your_napcat_token
```

### 3. 数据库初始化

确保PostgreSQL已安装并运行，然后执行：

```bash
# 创建数据库
createdb napcat_db

# 启用pgvector扩展
psql -d napcat_db -c "CREATE EXTENSION IF NOT EXISTS vector;"

# 运行数据库迁移
./init_db.sh
```

### 4. 构建和运行

```bash
# 构建项目
cargo build --release

# 运行服务
./run.sh
```

服务将在 `http://0.0.0.0:8082` 启动。

## 📖 使用方法

### 基本对话

在QQ群中直接@机器人或发送消息，机器人会自动回复：

```
用户: 你好，介绍一下自己
机器人: 你好！我是NapCat AI助手，一个智能聊天机器人...
```

### 知识库学习

使用 `/learn` 命令让机器人学习新知识：

```
用户: /learn Rust是一种系统编程语言，注重安全性和性能
机器人: 知识存储成功！

用户: 什么是Rust？
机器人: Rust是一种系统编程语言，注重安全性和性能...
```

### 数据库查询命令

系统支持通过QQ指令实时检查数据库内容：

**查看数据库统计信息**:
```
用户: /db_stats
机器人: 数据库统计信息:
- 对话记录数量: 15
- 知识库文档数量: 8
- 用户上下文数量: 3
- 群组上下文数量: 2
```

**查看最近对话记录**:
```
用户: /recent 5
机器人: 最近的5条对话记录:
1. 用户: 你好 - 机器人: 你好！
2. 用户: 今天天气如何？ - 机器人: 我无法获取实时天气...
...
```

**查看知识库文档**:
```
用户: /docs
机器人: 知识库文档列表:
- ID: 1, 创建时间: 2024-01-01, 内容: Rust是一种系统编程语言...
- ID: 2, 创建时间: 2024-01-02, 内容: 人工智能是计算机科学的一个分支...
...
```

**查看系统配置**:
```
用户: /config
机器人: 系统配置信息:
- LLM模型: gpt-3.5-turbo
- 最大历史长度: 10
- 相似度阈值: 0.6
...
```

**查看用户上下文**:
```
用户: /user_ctx
机器人: 用户上下文信息:
- 对话历史: [你好, 你好！, ...]
- 上下文长度: 5
...
```

**查看群组上下文**:
```
用户: /group_ctx
机器人: 群组上下文信息:
- 群组ID: 123456
- 对话历史: [用户A: 你好, 机器人: 你好！, ...]
...
```

### 管理API

系统提供RESTful API进行管理：

#### 获取系统信息
```bash
curl http://localhost:8082/api/system/info
```

#### 搜索知识库
```bash
curl -X POST http://localhost:8082/api/knowledge \
  -H "Content-Type: application/json" \
  -d '{"query": "Rust", "limit": 5}'
```

#### 添加知识
```bash
curl -X POST http://localhost:8082/api/knowledge \
  -H "Content-Type: application/json" \
  -d '{"content": "新的知识点内容"}'
```

更多API请参考 [API文档](API_DOCUMENTATION.md)。

## ⚙️ 配置说明

### 系统配置 (`config.toml`)

```toml
[system]
log_level = "info"                    # 日志级别
max_history_length = 10               # 最大对话历史长度
knowledge_search_limit = 5            # 知识库搜索数量限制
similarity_threshold = 0.6            # 相似度阈值

[prompts]
default_type = "Chat"                 # 默认提示词类型
enable_versioning = true              # 启用版本管理
```

### 提示词模板

系统支持多种提示词模板，可在配置文件中自定义：

- **Chat**: 通用对话模式
- **TechExpert**: 技术专家模式
- **CreativeWriter**: 创意写作模式
- **Analyst**: 专业分析模式
- **Friend**: 友善伙伴模式
- **Teacher**: 教学模式

### 环境变量

| 变量名 | 说明 | 示例 |
|--------|------|------|
| `DATABASE_URL` | PostgreSQL连接字符串 | `postgres://user:pass@localhost/napcat_db` |
| `LLM_API_KEY` | 大语言模型API密钥 | `sk-xxx...` |
| `LLM_BASE_URL` | LLM API基础URL | `https://api.openai.com/v1` |
| `LLM_MODEL` | 使用的模型名称 | `gpt-3.5-turbo` |
| `NAPCAT_TOKEN` | NapCat认证令牌 | `your_token` |

## 🛠️ 开发指南

### 项目结构

```
napcat_backend/
├── src/
│   ├── main.rs              # 主程序入口
│   ├── config.rs            # 配置管理
│   ├── db.rs                # 数据库操作
│   ├── models.rs            # 数据模型
│   ├── knowledge_base.rs    # 知识库实现
│   ├── context_manager.rs   # 上下文管理
│   ├── prompts.rs           # 提示词系统
│   └── enhanced_prompts.rs  # 增强提示词管理
├── migrations/              # 数据库迁移文件
├── config.toml             # 配置文件
└── Cargo.toml              # Rust项目配置
```

### 开发脚本

项目提供了多个开发脚本：

- `./dev.sh` - 开发模式运行
- `./run.sh` - 生产模式运行
- `./test.sh` - 运行测试
- `./check.sh` - 代码检查
- `./init_db.sh` - 数据库初始化

### 添加新功能

1. **添加新的提示词模板**：
   在 `config.toml` 的 `[prompts.custom_templates]` 部分添加新模板。

2. **扩展API接口**：
   在 `main.rs` 的 `Router::new()` 中添加新的路由和处理函数。

3. **添加数据库模型**：
   更新 `models.rs` 和相应的迁移文件。

## 🔍 故障排除

### 常见问题

**Q: 连接数据库失败**
A: 检查 `DATABASE_URL` 是否正确，确保PostgreSQL运行正常并已启用pgvector扩展。

**Q: LLM API调用失败**
A: 验证 `LLM_API_KEY` 和 `LLM_BASE_URL` 配置，确保API服务可访问。

**Q: NapCat连接失败**
A: 检查NapCat配置和网络连接，确认 `NAPCAT_TOKEN` 正确。

**Q: 知识库搜索无结果**
A: 确保已添加知识内容，检查 `similarity_threshold` 配置是否过高。

### 日志调试

设置 `RUST_LOG=debug` 环境变量获取详细日志：

```bash
RUST_LOG=debug ./run.sh
```

## 📚 相关文档

- [API文档](API_DOCUMENTATION.md) - 完整的API接口文档
- [提示词使用指南](PROMPTS_USAGE.md) - 提示词系统详细说明
- [脚本概述](SCRIPTS_OVERVIEW.md) - 开发脚本使用说明
- [快速开始](QUICK_START.md) - 快速部署指南

## 🤝 贡献指南

欢迎提交Issue和Pull Request！请确保：

1. 代码通过 `cargo check` 和 `cargo clippy` 检查
2. 添加适当的测试
3. 更新相关文档
4. 遵循Rust编码规范

## 📄 许可证

本项目采用MIT许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

- [NapCat](https://github.com/NapNeko/NapCatQQ) - QQ机器人框架
- [FastEmbed](https://github.com/Anush008/fastembed-rs) - 本地嵌入模型
- [Axum](https://github.com/tokio-rs/axum) - Web框架
- [SQLx](https://github.com/launchbadge/sqlx) - 异步SQL工具包

---

**⭐ 如果这个项目对你有帮助，请给个Star！**