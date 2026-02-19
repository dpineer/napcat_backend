# NapCat AI助手 - 管理API文档

## 概述

NapCat AI助手现在提供了一套完整的HTTP API，用于管理和配置系统。这些API支持跨域访问（CORS），可以被Flutter、Web或其他客户端应用程序调用。

## 基础信息

- **基础URL**: `http://localhost:8082/api`
- **认证**: 目前不需要认证（生产环境建议添加）
- **内容类型**: `application/json`
- **CORS**: 已启用，支持所有来源、方法和头部

## API端点

### 1. 系统配置管理

#### 获取系统配置
```http
GET /api/config
```

**响应示例**:
```json
{
  "status": "success",
  "data": {
    "llm_base_url": "https://api.openai.com/v1",
    "llm_model": "gpt-3.5-turbo",
    "prompt_stats": {
      "total_templates": 6,
      "available_types": ["Chat", "Learn", "Analyze", "Creative", "Professional", "Friendly"],
      "custom_templates": 0,
      "default_type": "标准聊天模式，适合日常对话",
      "config_loaded": false
    },
    "system_info": {
      "knowledge_search_limit": 5,
      "similarity_threshold": 0.6,
      "max_history_length": 10,
      "log_level": "info"
    }
  }
}
```

#### 更新系统配置
```http
POST /api/config
```

**请求体**:
```json
{
  "llm_base_url": "https://api.new-provider.com/v1",
  "llm_model": "gpt-4",
  "knowledge_search_limit": 8,
  "similarity_threshold": 0.7,
  "max_history_length": 15,
  "log_level": "debug"
}
```

**响应示例**:
```json
{
  "status": "success",
  "message": "配置更新已接收，需要重启服务以应用更改",
  "note": "在实际生产环境中，应该将配置保存到数据库或配置文件"
}
```

### 2. 知识库管理

#### 搜索知识库
```http
GET /api/knowledge
```

**请求体**:
```json
{
  "query": "Rust编程",
  "limit": 5
}
```

**响应示例**:
```json
{
  "status": "success",
  "data": {
    "query": "Rust编程",
    "results": [
      "Rust是一种系统编程语言...",
      "Rust的所有权系统...",
      "Rust的内存安全性..."
    ],
    "count": 3
  }
}
```

#### 添加知识
```http
POST /api/knowledge
```

**请求体**:
```json
{
  "content": "Rust是一种专注于安全性、速度和并发性的系统编程语言。它通过所有权系统来管理内存，无需垃圾回收器。"
}
```

**响应示例**:
```json
{
  "status": "success",
  "message": "知识添加成功"
}
```

### 3. 提示词管理

#### 获取提示词信息
```http
GET /api/prompts
```

**响应示例**:
```json
{
  "status": "success",
  "data": {
    "available_types": ["Chat", "Learn", "Analyze", "Creative", "Professional", "Friendly"],
    "stats": {
      "total_templates": 6,
      "available_types": ["Chat", "Learn", "Analyze", "Creative", "Professional", "Friendly"],
      "custom_templates": 0,
      "default_type": "标准聊天模式，适合日常对话",
      "config_loaded": false
    },
    "smart_selection_enabled": true
  }
}
```

#### 更新提示词配置
```http
POST /api/prompts
```

**请求体**:
```json
{
  "default_type": "Professional",
  "custom_templates": {
    "tech_expert": {
      "enabled": true,
      "system_prompt": "你是一个资深的技术专家...",
      "user_template": "技术背景知识：\n{knowledge}\n\n技术问题：{question}"
    }
  }
}
```

**响应示例**:
```json
{
  "status": "success",
  "message": "提示词配置更新成功",
  "note": "在实际生产环境中，应该实现具体的提示词更新逻辑"
}
```

### 4. 提示词统计

#### 获取提示词统计信息
```http
GET /api/prompts/stats
```

**响应示例**:
```json
{
  "status": "success",
  "data": {
    "total_templates": 6,
    "available_types": ["Chat", "Learn", "Analyze", "Creative", "Professional", "Friendly"],
    "custom_templates": 0,
    "default_type": "标准聊天模式，适合日常对话",
    "config_loaded": false
  }
}
```

### 5. 系统信息

#### 获取系统信息
```http
GET /api/system/info
```

**响应示例**:
```json
{
  "status": "success",
  "data": {
    "system": {
      "knowledge_search_limit": 5,
      "similarity_threshold": 0.6,
      "max_history_length": 10,
      "log_level": "info"
    },
    "llm": {
      "base_url": "https://api.openai.com/v1",
      "model": "gpt-3.5-turbo"
    },
    "features": {
      "smart_prompt_selection": true,
      "configurable_prompts": true,
      "knowledge_base": true,
      "conversation_context": true
    }
  }
}
```

## 智能提示词选择

系统会根据用户输入自动选择最合适的提示词模式：

- **分析模式**: 包含"分析"、"怎么看"等关键词
- **创意模式**: 包含"创意"、"想法"等关键词  
- **专业模式**: 包含"技术"、"编程"、"代码"等关键词
- **学习模式**: 包含"学习"、"记住"、"记录"等关键词
- **友好模式**: 包含"朋友"、"聊天"、"说说"等关键词
- **标准聊天**: 其他情况默认使用

## 错误处理

所有API都遵循统一的错误响应格式：

```json
{
  "status": "error",
  "message": "具体的错误信息"
}
```

常见的错误情况：
- 400 Bad Request: 请求参数无效
- 500 Internal Server Error: 服务器内部错误
- 数据库连接失败
- 知识库操作失败

## 使用示例

### Flutter 集成示例

```dart
import 'package:http/http.dart' as http;
import 'dart:convert';

class NapCatApi {
  final String baseUrl = 'http://localhost:8082/api';
  
  // 获取系统配置
  Future<Map<String, dynamic>> getConfig() async {
    final response = await http.get(Uri.parse('$baseUrl/config'));
    if (response.statusCode == 200) {
      return json.decode(response.body);
    } else {
      throw Exception('Failed to load config');
    }
  }
  
  // 搜索知识库
  Future<Map<String, dynamic>> searchKnowledge(String query, {int limit = 5}) async {
    final response = await http.post(
      Uri.parse('$baseUrl/knowledge'),
      headers: {'Content-Type': 'application/json'},
      body: json.encode({
        'query': query,
        'limit': limit,
      }),
    );
    
    if (response.statusCode == 200) {
      return json.decode(response.body);
    } else {
      throw Exception('Failed to search knowledge');
    }
  }
  
  // 添加知识
  Future<Map<String, dynamic>> addKnowledge(String content) async {
    final response = await http.post(
      Uri.parse('$baseUrl/knowledge'),
      headers: {'Content-Type': 'application/json'},
      body: json.encode({'content': content}),
    );
    
    if (response.statusCode == 200) {
      return json.decode(response.body);
    } else {
      throw Exception('Failed to add knowledge');
    }
  }
  
  // 获取提示词统计
  Future<Map<String, dynamic>> getPromptsStats() async {
    final response = await http.get(Uri.parse('$baseUrl/prompts/stats'));
    if (response.statusCode == 200) {
      return json.decode(response.body);
    } else {
      throw Exception('Failed to load prompts stats');
    }
  }
}
```

### JavaScript/TypeScript 示例

```javascript
class NapCatApi {
  constructor(baseUrl = 'http://localhost:8082/api') {
    this.baseUrl = baseUrl;
  }
  
  async getConfig() {
    const response = await fetch(`${this.baseUrl}/config`);
    if (!response.ok) throw new Error('Failed to fetch config');
    return await response.json();
  }
  
  async searchKnowledge(query, limit = 5) {
    const response = await fetch(`${this.baseUrl}/knowledge`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ query, limit }),
    });
    if (!response.ok) throw new Error('Failed to search knowledge');
    return await response.json();
  }
  
  async addKnowledge(content) {
    const response = await fetch(`${this.baseUrl}/knowledge`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ content }),
    });
    if (!response.ok) throw new Error('Failed to add knowledge');
    return await response.json();
  }
  
  async getPromptsStats() {
    const response = await fetch(`${this.baseUrl}/prompts/stats`);
    if (!response.ok) throw new Error('Failed to fetch prompts stats');
    return await response.json();
  }
}
```

### cURL 示例

```bash
# 获取系统配置
curl http://localhost:8082/api/config

# 搜索知识库
curl -X POST http://localhost:8082/api/knowledge \
  -H "Content-Type: application/json" \
  -d '{"query": "Rust编程", "limit": 3}'

# 添加知识
curl -X POST http://localhost:8082/api/knowledge \
  -H "Content-Type: application/json" \
  -d '{"content": "Rust是一种系统编程语言..."}'

# 获取提示词统计
curl http://localhost:8082/api/prompts/stats

# 获取系统信息
curl http://localhost:8082/api/system/info
```

## 安全考虑

### 生产环境建议

1. **认证授权**: 添加API密钥或JWT认证
2. **HTTPS**: 使用SSL/TLS加密通信
3. **输入验证**: 加强请求参数验证
4. **敏感信息**: 避免在响应中返回敏感配置（如API密钥）
5. **速率限制**: 实现API调用频率限制
6. **日志记录**: 记录所有API调用用于审计

### 当前实现的安全措施

- 敏感配置（如LLM API密钥）不会通过API返回
- 配置更新需要重启服务才能生效，防止运行时恶意修改
- 所有输入都进行了基本的空值检查
- 错误信息不会暴露内部实现细节

## 性能优化

### 建议的优化措施

1. **缓存**: 对频繁访问的数据实现缓存机制
2. **分页**: 对大量数据实现分页返回
3. **异步处理**: 对耗时操作使用异步处理
4. **数据库索引**: 优化数据库查询性能
5. **连接池**: 使用数据库连接池

## 监控和调试

### 日志记录

所有API调用都会记录到系统日志中，包括：
- 请求方法和路径
- 请求参数（敏感信息会被过滤）
- 响应状态
- 错误信息

### 健康检查

可以通过访问 `/api/system/info` 来检查系统状态，包括：
- 系统配置参数
- LLM服务状态
- 功能特性状态

## 版本历史

- v1.0.0: 初始API版本，包含基本的配置、知识库和提示词管理功能
- v1.1.0: 添加CORS支持，优化错误处理
- v1.2.0: 添加系统信息接口，完善统计功能

## 支持与反馈

如有问题或建议，请通过以下方式联系：
- 提交GitHub Issue
- 查看项目文档
- 参考示例代码

这个API文档将随着系统功能的扩展而持续更新。