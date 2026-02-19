use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

/// 提示词模板类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PromptType {
    /// 标准聊天提示词
    Chat,
    /// 学习模式提示词
    Learn,
    /// 分析模式提示词
    Analyze,
    /// 创意模式提示词
    Creative,
    /// 专业模式提示词
    Professional,
    /// 友好模式提示词
    Friendly,
}

/// 提示词模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    /// 系统提示词
    pub system_prompt: String,
    /// 用户提示词模板
    pub user_template: String,
    /// 模板描述
    pub description: String,
    /// 模板参数
    pub parameters: Vec<String>,
}

/// 提示词管理器
pub struct PromptManager {
    /// 提示词模板映射
    templates: HashMap<PromptType, PromptTemplate>,
    /// 默认提示词类型
    default_type: PromptType,
}

impl Default for PromptManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptManager {
    /// 创建新的提示词管理器
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
            default_type: PromptType::Chat,
        };
        
        // 初始化默认模板
        manager.init_default_templates();
        manager
    }

    /// 初始化默认提示词模板
    fn init_default_templates(&mut self) {
        // 标准聊天模板
        self.templates.insert(
            PromptType::Chat,
            PromptTemplate {
                system_prompt: "你是一个AI助手，专门为群聊提供智能回复。请使用提供的上下文信息来回答问题，如果不确定，请诚实地说出来。保持回答简洁、有用且友好。".to_string(),
                user_template: "知识库信息：\n{knowledge}\n\n对话历史：\n{history}\n\n用户问题：{question}".to_string(),
                description: "标准聊天模式，适合日常对话".to_string(),
                parameters: vec!["knowledge".to_string(), "history".to_string(), "question".to_string()],
            },
        );

        // 学习模式模板
        self.templates.insert(
            PromptType::Learn,
            PromptTemplate {
                system_prompt: "你是一个知识管理助手，帮助用户学习和记忆重要信息。当用户说'学习'时，请确认信息已被记录。当用户提问时，请优先使用已学习的知识来回答。".to_string(),
                user_template: "用户想要学习的内容：{content}\n\n请确认这条信息已被记录。".to_string(),
                description: "学习模式，用于记录和管理知识".to_string(),
                parameters: vec!["content".to_string()],
            },
        );

        // 分析模式模板
        self.templates.insert(
            PromptType::Analyze,
            PromptTemplate {
                system_prompt: "你是一个专业的数据分析师，擅长从对话中提取关键信息和洞察。请提供深入、客观的分析，并给出具体的建议。".to_string(),
                user_template: "请分析以下对话内容：\n\n对话历史：\n{history}\n\n当前问题：{question}\n\n请提供详细的分析和建议。".to_string(),
                description: "分析模式，提供深度分析".to_string(),
                parameters: vec!["history".to_string(), "question".to_string()],
            },
        );

        // 创意模式模板
        self.templates.insert(
            PromptType::Creative,
            PromptTemplate {
                system_prompt: "你是一个富有创意的AI助手，擅长头脑风暴和创新思考。请提供独特、有趣且富有想象力的回答，鼓励创造性思维。".to_string(),
                user_template: "知识背景：\n{knowledge}\n\n对话背景：\n{history}\n\n创意挑战：{question}\n\n请提供富有创意的回答或建议。".to_string(),
                description: "创意模式，激发创新思维".to_string(),
                parameters: vec!["knowledge".to_string(), "history".to_string(), "question".to_string()],
            },
        );

        // 专业模式模板
        self.templates.insert(
            PromptType::Professional,
            PromptTemplate {
                system_prompt: "你是一个专业的技术顾问，具有深厚的技术背景和丰富的实践经验。请提供准确、详细且专业的技术解答。".to_string(),
                user_template: "技术知识库：\n{knowledge}\n\n技术背景：\n{history}\n\n技术问题：{question}\n\n请提供专业的技术解答。".to_string(),
                description: "专业模式，提供技术专业解答".to_string(),
                parameters: vec!["knowledge".to_string(), "history".to_string(), "question".to_string()],
            },
        );

        // 友好模式模板
        self.templates.insert(
            PromptType::Friendly,
            PromptTemplate {
                system_prompt: "你是一个友善的AI伙伴，总是以温暖、鼓励和支持的方式与用户交流。请用轻松愉快的语调回答问题，让用户感到舒适和被理解。".to_string(),
                user_template: "相关知识：\n{knowledge}\n\n对话记录：\n{history}\n\n用户的问题：{question}\n\n请以友善温暖的方式回答。".to_string(),
                description: "友好模式，提供温暖的交流体验".to_string(),
                parameters: vec!["knowledge".to_string(), "history".to_string(), "question".to_string()],
            },
        );
    }

    /// 获取指定类型的提示词模板
    pub fn get_template(&self, prompt_type: &PromptType) -> Option<&PromptTemplate> {
        self.templates.get(prompt_type)
    }

    /// 获取默认提示词模板
    pub fn get_default_template(&self) -> &PromptTemplate {
        self.templates.get(&self.default_type)
            .expect("Default template should exist")
    }

    /// 设置默认提示词类型
    pub fn set_default_type(&mut self, prompt_type: PromptType) {
        self.default_type = prompt_type;
    }

    /// 添加自定义提示词模板
    pub fn add_template(&mut self, prompt_type: PromptType, template: PromptTemplate) {
        self.templates.insert(prompt_type, template);
    }

    /// 渲染提示词模板
    pub fn render_template(&self, prompt_type: &PromptType, parameters: &HashMap<String, String>) -> Result<(String, String)> {
        let template = self.get_template(prompt_type)
            .ok_or_else(|| anyhow::anyhow!("Template not found for type: {:?}", prompt_type))?;

        let system_prompt = template.system_prompt.clone();
        let mut user_prompt = template.user_template.clone();

        // 替换模板参数
        for (key, value) in parameters {
            let placeholder = format!("{{{}}}", key);
            user_prompt = user_prompt.replace(&placeholder, value);
        }

        Ok((system_prompt, user_prompt))
    }

    /// 渲染默认模板
    pub fn render_default(&self, parameters: &HashMap<String, String>) -> Result<(String, String)> {
        self.render_template(&self.default_type, parameters)
    }

    /// 获取所有可用的提示词类型
    pub fn get_available_types(&self) -> Vec<PromptType> {
        self.templates.keys().cloned().collect()
    }

    /// 获取提示词类型描述
    pub fn get_type_description(&self, prompt_type: &PromptType) -> Option<String> {
        self.templates.get(prompt_type).map(|t| t.description.clone())
    }
}

/// 聊天提示词构建器
pub struct ChatPromptBuilder {
    knowledge: Option<String>,
    history: Option<String>,
    question: String,
    prompt_type: PromptType,
}

impl ChatPromptBuilder {
    /// 创建新的聊天提示词构建器
    pub fn new(question: String) -> Self {
        Self {
            knowledge: None,
            history: None,
            question,
            prompt_type: PromptType::Chat,
        }
    }

    /// 设置知识库内容
    pub fn with_knowledge(mut self, knowledge: String) -> Self {
        self.knowledge = Some(knowledge);
        self
    }

    /// 设置对话历史
    pub fn with_history(mut self, history: String) -> Self {
        self.history = Some(history);
        self
    }

    /// 设置提示词类型
    pub fn with_prompt_type(mut self, prompt_type: PromptType) -> Self {
        self.prompt_type = prompt_type;
        self
    }

    /// 构建提示词
    pub fn build(self, prompt_manager: &PromptManager) -> Result<(String, String)> {
        let mut parameters = HashMap::new();
        
        parameters.insert("question".to_string(), self.question);
        parameters.insert("knowledge".to_string(), self.knowledge.unwrap_or_default());
        parameters.insert("history".to_string(), self.history.unwrap_or_default());

        prompt_manager.render_template(&self.prompt_type, &parameters)
    }
}

/// 学习提示词构建器
pub struct LearnPromptBuilder {
    content: String,
}

impl LearnPromptBuilder {
    /// 创建新的学习提示词构建器
    pub fn new(content: String) -> Self {
        Self { content }
    }

    /// 构建学习提示词
    pub fn build(self, prompt_manager: &PromptManager) -> Result<(String, String)> {
        let mut parameters = HashMap::new();
        parameters.insert("content".to_string(), self.content);

        prompt_manager.render_template(&PromptType::Learn, &parameters)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_manager_creation() {
        let manager = PromptManager::new();
        assert!(manager.get_template(&PromptType::Chat).is_some());
        assert!(manager.get_template(&PromptType::Learn).is_some());
    }

    #[test]
    fn test_template_rendering() {
        let manager = PromptManager::new();
        let mut parameters = HashMap::new();
        parameters.insert("knowledge".to_string(), "测试知识".to_string());
        parameters.insert("history".to_string(), "测试历史".to_string());
        parameters.insert("question".to_string(), "测试问题".to_string());

        let result = manager.render_template(&PromptType::Chat, &parameters);
        assert!(result.is_ok());
        
        let (system, user) = result.unwrap();
        assert!(!system.is_empty());
        assert!(user.contains("测试知识"));
        assert!(user.contains("测试历史"));
        assert!(user.contains("测试问题"));
    }

    #[test]
    fn test_chat_prompt_builder() {
        let manager = PromptManager::new();
        let builder = ChatPromptBuilder::new("测试问题".to_string())
            .with_knowledge("测试知识".to_string())
            .with_history("测试历史".to_string());

        let result = builder.build(&manager);
        assert!(result.is_ok());
    }
}