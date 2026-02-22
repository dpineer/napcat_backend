use crate::config::{AppConfig, ConfigManager as AppConfManager};
use std::collections::HashMap;
use anyhow::Result;
use tracing::{info, debug};
use serde::{Serialize, Deserialize};

// 定义缺失的类型
#[derive(Debug, Clone, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PromptType {
    Chat,
    Professional,
    Creative,
    Analyze,
    Learn,
    Friendly,
}

pub struct PromptManager {
    templates: HashMap<PromptType, PromptTemplate>,
    default_type: PromptType,
}

#[derive(Debug, Clone)]
pub struct PromptTemplate {
    pub description: String,
    pub system_prompt: String,
    pub user_prompt: String,
}

pub struct ChatPromptBuilder {
    question: String,
    knowledge: String,
    history: String,
    prompt_type: PromptType,
}

impl PromptManager {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        
        // 添加默认模板
        templates.insert(
            PromptType::Chat,
            PromptTemplate {
                description: "标准聊天".to_string(),
                system_prompt: "你是一个有用的助手。".to_string(),
                user_prompt: "{question}".to_string(),
            }
        );
        
        // 添加分析模板
        templates.insert(
            PromptType::Analyze,
            PromptTemplate {
                description: "深度分析".to_string(),
                system_prompt: "你是一个专业的分析助手，擅长深入分析问题并提供详细见解。".to_string(),
                user_prompt: "{question}".to_string(),
            }
        );
        
        // 添加创意模板
        templates.insert(
            PromptType::Creative,
            PromptTemplate {
                description: "创意写作".to_string(),
                system_prompt: "你是一个富有创意的写作助手，擅长提供创新想法和创意内容。".to_string(),
                user_prompt: "{question}".to_string(),
            }
        );
        
        // 添加专业模板
        templates.insert(
            PromptType::Professional,
            PromptTemplate {
                description: "专业助手".to_string(),
                system_prompt: "你是一个专业的技术助手，擅长解决技术问题和提供专业建议。".to_string(),
                user_prompt: "{question}".to_string(),
            }
        );
        
        // 添加学习模板
        templates.insert(
            PromptType::Learn,
            PromptTemplate {
                description: "学习助手".to_string(),
                system_prompt: "你是一个学习助手，专注于帮助用户学习和记忆新知识。".to_string(),
                user_prompt: "{question}".to_string(),
            }
        );
        
        // 添加友好模板
        templates.insert(
            PromptType::Friendly,
            PromptTemplate {
                description: "友好聊天".to_string(),
                system_prompt: "你是一个友好的聊天伙伴，善于进行轻松愉快的对话。".to_string(),
                user_prompt: "{question}".to_string(),
            }
        );
        
        Self {
            templates,
            default_type: PromptType::Chat,
        }
    }
    
    pub fn add_template(&mut self, prompt_type: PromptType, template: PromptTemplate) {
        self.templates.insert(prompt_type, template);
    }
    
    pub fn get_template(&self, prompt_type: &PromptType) -> Option<&PromptTemplate> {
        self.templates.get(prompt_type)
    }
    
    pub fn get_available_types(&self) -> Vec<PromptType> {
        self.templates.keys().cloned().collect()
    }
    
    pub fn get_default_template(&self) -> &PromptTemplate {
        self.templates.get(&self.default_type).unwrap()
    }
    
    pub fn set_default_type(&mut self, default_type: PromptType) {
        self.default_type = default_type;
    }
}

impl ChatPromptBuilder {
    pub fn new(question: String) -> Self {
        Self {
            question,
            knowledge: String::new(),
            history: String::new(),
            prompt_type: PromptType::Chat,
        }
    }
    
    pub fn with_knowledge(mut self, knowledge: String) -> Self {
        self.knowledge = knowledge;
        self
    }
    
    pub fn with_history(mut self, history: String) -> Self {
        self.history = history;
        self
    }
    
    pub fn with_prompt_type(mut self, prompt_type: PromptType) -> Self {
        self.prompt_type = prompt_type;
        self
    }
    
    pub fn build(&self, manager: &PromptManager) -> Result<(String, String)> {
        let template = manager.get_template(&self.prompt_type)
            .ok_or_else(|| anyhow::anyhow!("未找到模板类型: {:?}", self.prompt_type))?;
        
        let system_prompt = template.system_prompt.clone();
        let user_prompt = format!(
            "问题: {}\n\n知识: {}\n\n历史: {}",
            self.question,
            self.knowledge,
            self.history
        );
        
        Ok((system_prompt, user_prompt))
    }
}

/// 增强版提示词管理器
pub struct EnhancedPromptManager {
    /// 基础提示词管理器
    base_manager: PromptManager,
    /// 配置管理器
    config_manager: Option<crate::config::ConfigManager>,
    /// 自定义提示词类型映射
    custom_types: HashMap<String, PromptType>,
}

impl EnhancedPromptManager {
    /// 创建新的增强版提示词管理器
    pub fn new() -> Self {
        let mut manager = Self {
            base_manager: PromptManager::new(),
            config_manager: None,
            custom_types: HashMap::new(),
        };
        
        // 初始化自定义类型映射
        manager.init_custom_types();
        manager
    }

    /// 从配置创建增强版提示词管理器
    pub fn from_config(config_manager: crate::config::ConfigManager) -> Self {
        let mut manager = Self {
            base_manager: PromptManager::new(),
            config_manager: Some(config_manager),
            custom_types: HashMap::new(),
        };
        
        manager.init_custom_types();
        manager.load_config_templates();
        manager
    }

    /// 初始化自定义类型映射
    fn init_custom_types(&mut self) {
        // 为自定义模板创建唯一的提示词类型
        self.custom_types.insert("tech_expert".to_string(), PromptType::Professional);
        self.custom_types.insert("creative_writer".to_string(), PromptType::Creative);
        self.custom_types.insert("analyst".to_string(), PromptType::Analyze);
        self.custom_types.insert("friend".to_string(), PromptType::Friendly);
        self.custom_types.insert("learner".to_string(), PromptType::Learn);
    }

    /// 从配置加载自定义模板
    fn load_config_templates(&mut self) {
        if let Some(ref config_manager) = self.config_manager {
            let config = config_manager.get_config();
            
            // 设置默认提示词类型
            self.base_manager.set_default_type(config.prompts.default_type.clone());
            
            // 加载自定义模板
            for (name, custom_template) in &config.prompts.custom_templates {
                if custom_template.enabled {
                    let template = config.to_prompt_template(custom_template);
                    
                    // 根据名称映射到对应的提示词类型
                    if let Some(prompt_type) = self.custom_types.get(name) {
                        info!("加载自定义提示词模板: {} (类型: {:?})", name, prompt_type);
                        self.base_manager.add_template(prompt_type.clone(), template);
                    } else {
                        debug!("未找到自定义类型映射: {}，使用默认类型", name);
                        // 使用默认类型作为后备
                        self.base_manager.add_template(config.prompts.default_type.clone(), template);
                    }
                }
            }
        }
    }

    /// 获取基础提示词管理器
    pub fn get_base_manager(&self) -> &PromptManager {
        &self.base_manager
    }

    /// 获取可变的基础提示词管理器
    pub fn get_base_manager_mut(&mut self) -> &mut PromptManager {
        &mut self.base_manager
    }

    /// 获取配置管理器
    pub fn get_config_manager(&self) -> Option<&crate::config::ConfigManager> {
        self.config_manager.as_ref()
    }

    /// 获取系统配置
    pub fn get_system_config(&self) -> Option<&crate::config::SystemConfig> {
        self.config_manager.as_ref().map(|cm| &cm.get_config().system)
    }

    /// 重新加载配置
    pub fn reload_config(&mut self) -> Result<()> {
        if let Some(ref mut config_manager) = self.config_manager {
            config_manager.reload()?;
            
            // 重新加载提示词模板
            self.base_manager = PromptManager::new(); // 重置基础管理器
            self.load_config_templates();
            
            info!("配置已重新加载");
        }
        Ok(())
    }

    /// 更新配置
    pub fn update_config<F>(&mut self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut AppConfig),
    {
        if let Some(ref mut config_manager) = self.config_manager {
            config_manager.update_config(updater)?;
            
            // 重新加载提示词模板
            self.base_manager = PromptManager::new(); // 重置基础管理器
            self.load_config_templates();
            
            info!("配置已更新并重新加载");
        }
        Ok(())
    }

    /// 获取知识库搜索限制
    pub fn get_knowledge_search_limit(&self) -> usize {
        self.get_system_config()
            .map(|sc| sc.knowledge_search_limit)
            .unwrap_or(3) // 默认值
    }

    /// 获取相似度阈值
    pub fn get_similarity_threshold(&self) -> f32 {
        self.get_system_config()
            .map(|sc| sc.similarity_threshold)
            .unwrap_or(0.6) // 默认值
    }

    /// 获取最大历史长度
    pub fn get_max_history_length(&self) -> usize {
        self.get_system_config()
            .map(|sc| sc.max_history_length)
            .unwrap_or(5) // 默认值
    }

    /// 获取日志级别
    pub fn get_log_level(&self) -> String {
        self.get_system_config()
            .map(|sc| sc.log_level.clone())
            .unwrap_or("info".to_string())
    }

    /// 智能选择提示词类型
    pub fn smart_select_prompt_type(&self, message: &str) -> PromptType {
        let message_lower = message.to_lowercase();
        
        // 根据消息内容智能选择提示词类型
        if message_lower.contains("分析") || message_lower.contains("分析下") || message_lower.contains("怎么看") {
            // 检查Analyze类型是否存在，如果不存在则返回Chat
            if self.base_manager.get_template(&PromptType::Analyze).is_some() {
                PromptType::Analyze
            } else {
                PromptType::Chat
            }
        } else if message_lower.contains("创意") || message_lower.contains("想法") || message_lower.contains("创意") {
            // 检查Creative类型是否存在，如果不存在则返回Chat
            if self.base_manager.get_template(&PromptType::Creative).is_some() {
                PromptType::Creative
            } else {
                PromptType::Chat
            }
        } else if message_lower.contains("技术") || message_lower.contains("编程") || message_lower.contains("代码") {
            // 检查Professional类型是否存在，如果不存在则返回Chat
            if self.base_manager.get_template(&PromptType::Professional).is_some() {
                PromptType::Professional
            } else {
                PromptType::Chat
            }
        } else if message_lower.contains("学习") || message_lower.contains("记住") || message_lower.contains("记录") {
            // 检查Learn类型是否存在，如果不存在则返回Chat
            if self.base_manager.get_template(&PromptType::Learn).is_some() {
                PromptType::Learn
            } else {
                PromptType::Chat
            }
        } else if message_lower.contains("朋友") || message_lower.contains("聊天") || message_lower.contains("说说") {
            // 检查Friendly类型是否存在，如果不存在则返回Chat
            if self.base_manager.get_template(&PromptType::Friendly).is_some() {
                PromptType::Friendly
            } else {
                PromptType::Chat
            }
        } else {
            // 使用配置的默认类型
            PromptType::Chat
        }
    }

    /// 构建智能提示词
    pub fn build_smart_prompt(
        &self,
        question: String,
        knowledge: Option<String>,
        history: Option<String>,
    ) -> Result<(String, String)> {
        // 智能选择提示词类型
        let prompt_type = self.smart_select_prompt_type(&question);
        
        debug!("智能选择提示词类型: {:?} (问题: {})", prompt_type, question);
        
        // 构建提示词
        ChatPromptBuilder::new(question)
            .with_knowledge(knowledge.unwrap_or_default())
            .with_history(history.unwrap_or_default())
            .with_prompt_type(prompt_type)
            .build(&self.base_manager)
    }

    /// 获取提示词统计信息
    pub fn get_stats(&self) -> PromptStats {
        let available_types = self.base_manager.get_available_types();
        let custom_template_count = if let Some(ref config_manager) = self.config_manager {
            config_manager.get_config().prompts.custom_templates.len()
        } else {
            0
        };

        PromptStats {
            total_templates: available_types.len(),
            available_types: available_types.iter().map(|t| format!("{:?}", t)).collect(),
            custom_templates: custom_template_count,
            default_type: format!("{:?}", self.base_manager.get_default_template().description),
            config_loaded: self.config_manager.is_some(),
        }
    }
}

impl Default for EnhancedPromptManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 提示词统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptStats {
    pub total_templates: usize,
    pub available_types: Vec<String>,
    pub custom_templates: usize,
    pub default_type: String,
    pub config_loaded: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_prompt_manager_creation() {
        let manager = EnhancedPromptManager::new();
        assert!(manager.get_base_manager().get_template(&PromptType::Chat).is_some());
    }

    #[test]
    fn test_smart_select_prompt_type() {
        let manager = EnhancedPromptManager::new();
        
        assert_eq!(manager.smart_select_prompt_type("请分析这个问题"), PromptType::Analyze);
        assert_eq!(manager.smart_select_prompt_type("有什么创意想法吗"), PromptType::Creative);
        assert_eq!(manager.smart_select_prompt_type("这个技术问题怎么解决"), PromptType::Professional);
        assert_eq!(manager.smart_select_prompt_type("学习这个知识"), PromptType::Learn);
        assert_eq!(manager.smart_select_prompt_type("朋友，聊聊吧"), PromptType::Friendly);
    }

    #[test]
    fn test_build_smart_prompt() {
        let manager = EnhancedPromptManager::new();
        
        let result = manager.build_smart_prompt(
            "请分析这个问题".to_string(),
            Some("一些知识".to_string()),
            Some("一些历史".to_string()),
        );
        
        assert!(result.is_ok());
        let (system, user) = result.unwrap();
        assert!(!system.is_empty());
        assert!(user.contains("一些知识"));
        assert!(user.contains("一些历史"));
    }
}