use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use anyhow::Result;
use crate::prompts::{PromptTemplate, PromptType};

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 提示词配置
    pub prompts: PromptConfig,
    /// 系统配置
    pub system: SystemConfig,
}

/// 提示词配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptConfig {
    /// 默认提示词类型
    pub default_type: PromptType,
    /// 自定义提示词模板
    pub custom_templates: HashMap<String, CustomPromptTemplate>,
    /// 是否启用提示词版本管理
    pub enable_versioning: bool,
    /// 提示词配置文件路径
    pub config_path: Option<String>,
}

/// 自定义提示词模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPromptTemplate {
    /// 系统提示词
    pub system_prompt: String,
    /// 用户提示词模板
    pub user_template: String,
    /// 模板描述
    pub description: String,
    /// 模板参数
    pub parameters: Vec<String>,
    /// 模板版本
    pub version: String,
    /// 是否启用
    pub enabled: bool,
}

/// 系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// 日志级别
    pub log_level: String,
    /// 最大对话历史记录数
    pub max_history_length: usize,
    /// 知识库搜索结果数量
    pub knowledge_search_limit: usize,
    /// 相似度阈值
    pub similarity_threshold: f32,
    /// 是否启用自动学习功能
    pub auto_learn_enabled: bool,
    /// 是否学习发言格式（用户名、时间等）
    pub learn_message_format: bool,
    /// 自动学习的最小内容长度
    pub auto_learn_min_length: usize,
    /// 自动学习的最大内容长度
    pub auto_learn_max_length: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl AppConfig {
    /// 创建默认配置
    pub fn new() -> Self {
        Self {
            prompts: PromptConfig {
                default_type: PromptType::Chat,
                custom_templates: HashMap::new(),
                enable_versioning: false,
                config_path: None,
            },
            system: SystemConfig {
                log_level: "info".to_string(),
                max_history_length: 10,
                knowledge_search_limit: 5,
                similarity_threshold: 0.6,
                auto_learn_enabled: true,
                learn_message_format: false,
                auto_learn_min_length: 10,
                auto_learn_max_length: 2000,
            },
        }
    }

    /// 从文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self> {
        let mut config = Self::new();

        // 加载系统配置
        if let Ok(log_level) = std::env::var("LOG_LEVEL") {
            config.system.log_level = log_level;
        }

        if let Ok(max_history) = std::env::var("MAX_HISTORY_LENGTH") {
            if let Ok(length) = max_history.parse::<usize>() {
                config.system.max_history_length = length;
            }
        }

        if let Ok(search_limit) = std::env::var("KNOWLEDGE_SEARCH_LIMIT") {
            if let Ok(limit) = search_limit.parse::<usize>() {
                config.system.knowledge_search_limit = limit;
            }
        }

        if let Ok(threshold) = std::env::var("SIMILARITY_THRESHOLD") {
            if let Ok(thresh) = threshold.parse::<f32>() {
                config.system.similarity_threshold = thresh;
            }
        }

        // 加载提示词配置
        if let Ok(default_type) = std::env::var("DEFAULT_PROMPT_TYPE") {
            match default_type.as_str() {
                "chat" => config.prompts.default_type = PromptType::Chat,
                "learn" => config.prompts.default_type = PromptType::Learn,
                "analyze" => config.prompts.default_type = PromptType::Analyze,
                "creative" => config.prompts.default_type = PromptType::Creative,
                "professional" => config.prompts.default_type = PromptType::Professional,
                "friendly" => config.prompts.default_type = PromptType::Friendly,
                _ => {}
            }
        }

        if let Ok(enable_versioning) = std::env::var("ENABLE_PROMPT_VERSIONING") {
            config.prompts.enable_versioning = enable_versioning.parse().unwrap_or(false);
        }

        Ok(config)
    }

    /// 获取自定义提示词模板
    pub fn get_custom_template(&self, name: &str) -> Option<&CustomPromptTemplate> {
        self.prompts.custom_templates.get(name)
    }

    /// 添加自定义提示词模板
    pub fn add_custom_template(&mut self, name: String, template: CustomPromptTemplate) {
        self.prompts.custom_templates.insert(name, template);
    }

    /// 转换为标准提示词模板
    pub fn to_prompt_template(&self, custom: &CustomPromptTemplate) -> PromptTemplate {
        PromptTemplate {
            system_prompt: custom.system_prompt.clone(),
            user_template: custom.user_template.clone(),
            description: custom.description.clone(),
            parameters: custom.parameters.clone(),
        }
    }
}

/// 配置管理器
pub struct ConfigManager {
    config: AppConfig,
    config_path: Option<String>,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Self {
        Self {
            config: AppConfig::new(),
            config_path: None,
        }
    }

    /// 从文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = AppConfig::from_file(&path)?;
        let config_path = Some(path.as_ref().to_string_lossy().to_string());
        
        Ok(Self {
            config,
            config_path,
        })
    }

    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self> {
        let config = AppConfig::from_env()?;
        Ok(Self {
            config,
            config_path: None,
        })
    }

    /// 获取配置
    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }

    /// 获取可变配置
    pub fn get_config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    /// 保存配置
    pub fn save(&self) -> Result<()> {
        if let Some(ref path) = self.config_path {
            self.config.save_to_file(path)?;
        }
        Ok(())
    }

    /// 重新加载配置
    pub fn reload(&mut self) -> Result<()> {
        if let Some(ref path) = self.config_path {
            self.config = AppConfig::from_file(path)?;
        }
        Ok(())
    }

    /// 更新配置
    pub fn update_config<F>(&mut self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut AppConfig),
    {
        updater(&mut self.config);
        self.save()?;
        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 创建默认配置文件
pub fn create_default_config_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let config = AppConfig::new();
    
    // 添加一些示例自定义模板
    let mut config_with_examples = config;
    
    config_with_examples.add_custom_template(
        "tech_expert".to_string(),
        CustomPromptTemplate {
            system_prompt: "你是一个资深的技术专家，拥有丰富的编程和系统架构经验。请提供准确、详细的技术解答，并给出最佳实践建议。".to_string(),
            user_template: "技术背景知识：\n{knowledge}\n\n相关技术历史：\n{history}\n\n技术问题：{question}\n\n请提供专业的技术分析和解决方案。".to_string(),
            description: "技术专家模式，提供深度的技术分析".to_string(),
            parameters: vec!["knowledge".to_string(), "history".to_string(), "question".to_string()],
            version: "1.0.0".to_string(),
            enabled: true,
        },
    );

    config_with_examples.add_custom_template(
        "creative_writer".to_string(),
        CustomPromptTemplate {
            system_prompt: "你是一个富有创意的作家，擅长各种文体创作。请用生动、有趣的语言回答问题，激发用户的想象力和创造力。".to_string(),
            user_template: "创作背景知识：\n{knowledge}\n\n创作语境：\n{history}\n\n创作需求：{question}\n\n请以富有创意的方式回应。".to_string(),
            description: "创意写作模式，激发创作灵感".to_string(),
            parameters: vec!["knowledge".to_string(), "history".to_string(), "question".to_string()],
            version: "1.0.0".to_string(),
            enabled: true,
        },
    );

    config_with_examples.save_to_file(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = AppConfig::new();
        assert_eq!(config.system.log_level, "info");
        assert_eq!(config.system.max_history_length, 10);
    }

    #[test]
    fn test_custom_template() {
        let mut config = AppConfig::new();
        let template = CustomPromptTemplate {
            system_prompt: "Test system".to_string(),
            user_template: "Test user {param}".to_string(),
            description: "Test template".to_string(),
            parameters: vec!["param".to_string()],
            version: "1.0.0".to_string(),
            enabled: true,
        };

        config.add_custom_template("test".to_string(), template.clone());
        
        let retrieved = config.get_custom_template("test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().system_prompt, "Test system");
    }
}