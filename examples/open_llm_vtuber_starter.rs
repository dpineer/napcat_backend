use napcat_backend::*;
use std::sync::Arc;
use anyhow::Result;
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 启动 Open-LLM-VTuber 系统...");
    
    // 初始化配置
    let config_manager = DefaultConfigManager;
    let config = config_manager.default_config();
    
    println!("📋 使用配置:");
    println!("   - 角色名称: {}", config.character_config.name);
    println!("   - ASR提供者: {}", config.character_config.asr_config.provider);
    println!("   - TTS提供者: {}", config.character_config.tts_config.provider);
    println!("   - VAD提供者: {}", config.character_config.vad_config.provider);
    println!("   - LLM提供者: {}", config.character_config.agent_config.llm_provider);
    
    // 创建工厂实例
    let asr_factory = DefaultASRFactory;
    let tts_factory = DefaultTTSFactory;
    let vad_factory = DefaultVADFactory;
    let agent_factory = DefaultAgentFactory;
    let llm_factory = DefaultLLMFactory;
    
    // 创建服务上下文
    let mut service_context = ServiceContext::new();
    
    println!("⚙️  初始化各组件...");
    
    // 初始化ASR引擎
    let asr_engine = asr_factory.create_asr(&config.character_config.asr_config)?;
    service_context.init_asr(asr_engine);
    println!("✅ ASR引擎已初始化");
    
    // 初始化TTS引擎
    let tts_engine = tts_factory.create_tts(&config.character_config.tts_config)?;
    service_context.init_tts(tts_engine);
    println!("✅ TTS引擎已初始化");
    
    // 初始化VAD引擎
    let vad_engine = vad_factory.create_vad(&config.character_config.vad_config)?;
    service_context.init_vad(vad_engine);
    println!("✅ VAD引擎已初始化");
    
    // 初始化Agent引擎
    let agent_engine = agent_factory.create_agent(&config.character_config.agent_config)?;
    service_context.init_agent(agent_engine);
    println!("✅ Agent引擎已初始化");
    
    // 初始化LLM引擎
    let llm_engine = llm_factory.create_llm(&config)?;
    service_context.init_llm(llm_engine);
    println!("✅ LLM引擎已初始化");
    
    // 初始化MCP客户端
    let mcp_client = Arc::new(MCPClient::new());
    service_context.init_mcp(mcp_client);
    println!("✅ MCP客户端已初始化");
    
    println!("🌐 启动WebSocket处理器...");
    let ws_handler = WebSocketHandler::new();
    
    // 模拟一个完整的VTuber交互流程
    println!("\n🎭 模拟VTuber交互流程开始...");
    
    // 1. 模拟语音输入
    println!("\n🎤 模拟接收语音输入...");
    let asr = service_context.get_asr()?;
    let mock_audio = vec![0.1, 0.2, 0.3, 0.4, 0.5]; // 模拟音频数据
    let transcription = asr.transcribe_np(mock_audio)?;
    println!("📝 语音转文字结果: {}", transcription);
    
    // 2. 模拟用户输入文本
    let user_input = "你好，很高兴见到你！".to_string();
    println!("\n💬 用户输入: {}", user_input);
    
    // 3. 通过Agent处理输入
    println!("🤖 Agent处理输入...");
    let agent = service_context.get_agent()?;
    let input = BaseInput {
        text: user_input,
        audio_path: None,
        image_data: None,
    };
    
    let mut response_stream = agent.chat(input).await?;
    let mut agent_response = String::new();
    
    while let Some(response) = response_stream.next().await {
        match response {
            Ok(output) => {
                println!("🤖 Agent响应: {}", output.text);
                agent_response = output.text.clone();
                
                // 如果有音频路径，模拟播放
                if let Some(audio_path) = output.audio_path {
                    println!("🔊 音频输出路径: {}", audio_path);
                }
                
                // 如果有动作，模拟执行
                if let Some(actions) = output.actions {
                    for action in actions {
                        println!("🎭 执行动作: {}", action);
                    }
                }
            }
            Err(e) => eprintln!("❌ Agent处理错误: {}", e),
        }
    }
    
    // 4. 使用TTS生成语音输出
    if !agent_response.is_empty() {
        println!("\n🔊 使用TTS生成语音输出...");
        let tts = service_context.get_tts()?;
        let audio_file = tts.generate_audio(&agent_response, Some("vtuber_response"))?;
        println!("💾 生成音频文件: {}", audio_file);
    }
    
    // 5. 模拟VAD检测
    println!("\n🔍 使用VAD检测语音活动...");
    let vad = service_context.get_vad()?;
    let mock_audio_bytes = vec![0u8; 100]; // 模拟音频字节数据
    let speech_segments = vad.detect_speech(&mock_audio_bytes)?;
    println!("📊 检测到 {} 个语音片段", speech_segments.len());
    
    // 6. 模拟MCP工具调用
    println!("\n🔧 测试MCP工具调用...");
    let mcp = service_context.get_mcp()?;
    
    let server_config = ServerConfig {
        command: "echo".to_string(),
        args: vec!["MCP test".to_string()],
        env: None,
        cwd: None,
        timeout: Some(5),
    };
    
    mcp.add_server("test_server".to_string(), server_config).await?;
    println!("✅ 添加MCP服务器: test_server");
    
    let tools = mcp.list_tools("test_server").await?;
    println!("📦 服务器工具数量: {}", tools.len());
    
    println!("\n🎉 Open-LLM-VTuber 系统联动演示完成！");
    println!("✨ 所有组件已成功协同工作");
    
    // 模拟系统运行
    println!("\n⏳ 系统保持运行状态 (按 Ctrl+C 退出)...");
    tokio::signal::ctrl_c().await?;
    println!("\n🛑 系统已停止");
    
    Ok(())
}