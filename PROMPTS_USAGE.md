# NapCat AI助手 - 提示词系统使用说明

## 概述

NapCat AI助手现在具备了强大的提示词管理系统，支持多种对话模式和自定义配置。本系统允许您：

- 使用不同的AI对话模式（聊天、学习、分析、创意、专业、友好）
- 自定义提示词模板
- 通过配置文件管理提示词
- 智能模式选择
- 版本控制和A/B测试

## 系统架构

### 核心模块

1. **prompts.rs** - 基础提示词管理
2. **enhanced_prompts.rs** - 增强版提示词管理器
3. **config.rs** - 配置文件管理
4. **config.example.toml** - 配置文件示例

### 提示词类型

系统内置以下6种提示词类型：

| 类型 | 描述 | 适用场景 |
|------|------|----------|
| Chat | 标准聊天模式 | 日常对话、通用问答 |
| Learn | 学习模式 | 知识记录、学习辅导 |
| Analyze | 分析模式 | 数据分析、问题诊断 |
| Creative | 创意模式 | 头脑风暴、创意写作 |
| Professional | 专业模式 | 技术问题、专业咨询 |
| Friendly | 友好模式 | 情感支持、友好交流 |

## 智能模式选择

系统会根据用户输入自动选择最合适的提示词模式：

- 包含"分析"、"怎么看" → 分析模式
- 包含"创意"、"想法" → 创意模式
- 包含"技术"、"编程"、"代码" → 专业模式
- 包含"学习"、"记住"、"记录" → 学习模式
- 包含"朋友"、"聊天"、"说说" → 友好模式
- 其他情况 → 标准聊天模式

## 配置文件使用

### 1. 创建配置文件

复制示例配置文件：
```bash
cp config.example.toml config.toml
```

### 2. 配置文件结构

```toml
[system]
log_level = "info"                    # 日志级别
max_history_length = 10               # 最大历史记录数
knowledge_search_limit = 5            # 知识库搜索数量
similarity_threshold = 0.6            # 相似度阈值

[prompts]
default_type = "Chat"                 # 默认提示词类型
enable_versioning = true              # 启用版本管理

[prompts.custom_templates.模板名称]
system_prompt = "系统提示词..."
user_template = "用户模板..."
description = "模板描述"
parameters = ["knowledge", "history", "question"]
version = "1.0.0"
enabled = true
```

### 3. 自定义模板参数

模板支持以下参数：
- `{knowledge}` - 知识库内容
- `{history}` - 对话历史
- `{question}` - 用户问题
- `{content}` - 学习内容（学习模式专用）

## 使用示例

### 基本使用

```rust
use enhanced_prompts::EnhancedPromptManager;

// 创建提示词管理器
let prompt_manager = EnhancedPromptManager::new();

// 构建智能提示词
let (system_prompt, user_prompt) = prompt_manager.build_smart_prompt(
    "请分析这个数据".to_string(),
    Some("相关背景知识...".to_string()),
    Some("历史对话...".to_string()),
)?;
```

### 使用配置文件

```rust
use config::ConfigManager;
use enhanced_prompts::EnhancedPromptManager;

// 从配置文件加载
let config_manager = ConfigManager::from_file("config.toml")?;
let prompt_manager = EnhancedPromptManager::from_config(config_manager);
```

### 手动选择模式

```rust
use prompts::{ChatPromptBuilder, PromptType};

let (system, user) = ChatPromptBuilder::new("用户问题".to_string())
    .with_knowledge("知识库内容".to_string())
    .with_history("对话历史".to_string())
    .with_prompt_type(PromptType::Professional)
    .build(&prompt_manager.get_base_manager())?;
```

## 高级功能

### 1. 动态配置更新

```rust
// 更新配置
prompt_manager.update_config(|config| {
    config.system.log_level = "debug".to_string();
    config.prompts.default_type = PromptType::Analyze;
})?;
```

### 2. 配置重载

```rust
// 重新加载配置文件
prompt_manager.reload_config()?;
```

### 3. 获取统计信息

```rust
let stats = prompt_manager.get_stats();
println!("可用模板数: {}", stats.total_templates);
println!("自定义模板数: {}", stats.custom_templates);
println!("配置加载状态: {}", stats.config_loaded);
```

### 4. 智能参数获取

```rust
// 获取系统配置参数
let search_limit = prompt_manager.get_knowledge_search_limit();
let threshold = prompt_manager.get_similarity_threshold();
let max_history = prompt_manager.get_max_history_length();
let log_level = prompt_manager.get_log_level();
```

## 环境变量配置

支持通过环境变量进行配置：

- `LOG_LEVEL` - 日志级别
- `MAX_HISTORY_LENGTH` - 最大历史记录数
- `KNOWLEDGE_SEARCH_LIMIT` - 知识库搜索数量
- `SIMILARITY_THRESHOLD` - 相似度阈值
- `DEFAULT_PROMPT_TYPE` - 默认提示词类型
- `ENABLE_PROMPT_VERSIONING` - 启用版本管理

## 扩展开发

### 添加新的提示词类型

1. 在 `PromptType` 枚举中添加新类型
2. 在 `PromptManager::init_default_templates()` 中添加对应模板
3. 在 `EnhancedPromptManager::init_custom_types()` 中添加映射
4. 在智能选择逻辑中添加判断条件

### 创建自定义模板

1. 在配置文件中添加新的模板定义
2. 确保模板包含必要的参数（knowledge, history, question）
3. 设置合适的系统提示词和用户模板
4. 启用模板并设置版本号

## 最佳实践

### 1. 提示词设计原则

- **清晰性**：系统提示词要明确AI的角色和任务
- **具体性**：用户模板要包含具体的指令和格式要求
- **灵活性**：模板参数要设计合理，便于动态替换
- **一致性**：保持不同模式间的风格一致性

### 2. 性能优化

- 合理设置知识库搜索数量（3-5个结果最佳）
- 控制对话历史长度（5-10条记录）
- 设置适当的相似度阈值（0.5-0.7）

### 3. 用户体验

- 根据用户输入智能选择最合适的模式
- 提供清晰的模式切换指令
- 保持对话的连贯性和上下文一致性

## 故障排除

### 常见问题

1. **配置文件加载失败**
   - 检查文件路径和格式
   - 确保TOML语法正确
   - 验证所有必需字段

2. **提示词模板渲染失败**
   - 检查模板参数是否正确
   - 确保参数名称与模板中的占位符匹配
   - 验证模板语法

3. **智能模式选择不准确**
   - 检查关键词匹配逻辑
   - 考虑添加更多关键词
   - 调整匹配优先级

### 调试技巧

1. 启用debug日志级别查看详细信息
2. 使用 `get_stats()` 查看系统状态
3. 检查配置文件是否正确加载
4. 验证模板参数替换是否正确

## 版本历史

- v1.0.0 - 基础提示词系统
- v1.1.0 - 增强版提示词管理器
- v1.2.0 - 配置文件支持
- v1.3.0 - 智能模式选择
- v1.4.0 - 版本管理和A/B测试支持

## 贡献指南

欢迎贡献新的提示词模板和功能：

1. Fork项目并创建特性分支
2. 添加新的提示词类型或模板
3. 编写测试用例
4. 更新文档
5. 提交Pull Request

## 许可证

本项目采用MIT许可证，详见LICENSE文件。