use napcat_backend::enhanced_prompts::EnhancedPromptManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建增强版提示词管理器
    let manager = EnhancedPromptManager::new();
    
    // 测试智能选择提示词类型
    let test_messages = vec![
        "请分析这个问题的解决方案",
        "给我一些创意想法",
        "这个技术问题怎么解决",
        "帮我学习这个知识点",
        "朋友，聊聊今天的天气",
        "这是一个普通问题"
    ];
    
    println!("=== 智能提示词类型选择测试 ===");
    for message in test_messages {
        let prompt_type = manager.smart_select_prompt_type(message);
        println!("消息: \"{}\" -> 类型: {:?}", message, prompt_type);
    }
    
    println!("\n=== 智能提示词构建测试 ===");
    // 测试构建智能提示词
    let (system_prompt, user_prompt) = manager.build_smart_prompt(
        "请分析人工智能的发展趋势".to_string(),
        Some("AI技术正在快速发展".to_string()),
        Some("之前的对话历史".to_string()),
    )?;
    
    println!("系统提示词: {}", system_prompt);
    println!("用户提示词: {}", user_prompt);
    
    // 获取统计信息
    let stats = manager.get_stats();
    println!("\n=== 提示词管理器统计 ===");
    println!("模板总数: {}", stats.total_templates);
    println!("可用类型: {:?}", stats.available_types);
    println!("自定义模板数: {}", stats.custom_templates);
    println!("默认类型: {}", stats.default_type);
    println!("配置已加载: {}", stats.config_loaded);
    
    Ok(())
}