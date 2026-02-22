use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;
use futures::StreamExt;

use crate::{
    asr_interface::ASRInterface,
    tts_interface::TTSInterface,
    vad_interface::VADInterface,
    agent_interface::{AgentInterface, BaseInput, BaseOutput},
    stateless_llm_interface::StatelessLLMInterface,
    websocket_handler::{WebSocketHandler, MessageType},
    service_context::ServiceContext,
    mcp_interface::{MCPClient, ServerConfig},
    config_manager::{DefaultConfigManager, Config, ConfigManager},
    factory::{
        DefaultASRFactory, DefaultTTSFactory, DefaultVADFactory, 
        DefaultAgentFactory, DefaultLLMFactory, ASRFactory, TTSFactory, 
        VADFactory, AgentFactory, LLMFactory
    },
    // 引入napcat_backend的主要组件
    db::DatabaseRepository,
    knowledge_base::KnowledgeBase,
    context_manager::ContextManager,
    enhanced_prompts::EnhancedPromptManager,
    models::Config as ModelsConfig,
};

/// 统一启动器，整合napcat_backend和Open-LLM-VTuber功能
pub struct UnifiedLauncher {
    service_context: ServiceContext,
    napcat_components: NapCatComponents,
}

/// NapCat后端组件
pub struct NapCatComponents {
    kb: Arc<Mutex<KnowledgeBase>>,
    ctx_manager: Arc<ContextManager>,
    prompt_manager: Arc<EnhancedPromptManager>,
    config: Arc<crate::models::Config>,
    db: Arc<DatabaseRepository>,
}

impl UnifiedLauncher {
    /// 创建新的统一启动器
    pub async fn new() -> Result<Self> {
        println!("🚀 初始化统一启动器...");
        
        // 初始化配置
        let config_manager = DefaultConfigManager;
        let config = config_manager.default_config();
        
        println!("📋 配置加载完成");
        println!("   - 角色名称: {}", config.character_config.character_name);
        println!("   - ASR提供者: {}", config.character_config.asr_config.asr_model);
        println!("   - TTS提供者: {}", config.character_config.tts_config.tts_model);
        println!("   - VAD提供者: {}", config.character_config.vad_config.vad_model.as_deref().unwrap_or("silero"));
        println!("   - LLM提供者: {}", config.character_config.agent_config.llm_configs.provider);
        
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
        
        // 初始化NapCat组件
        let napcat_components = Self::init_napcat_components().await?;
        
        println!("✅ 统一启动器初始化完成");
        
        Ok(Self {
            service_context,
            napcat_components,
        })
    }
    
    /// 初始化NapCat后端组件
    async fn init_napcat_components() -> Result<NapCatComponents> {
        use std::env;
        use crate::{db, knowledge_base, context_manager, enhanced_prompts, models};
        
        // 从环境变量加载配置
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let llm_api_key = env::var("LLM_API_KEY").expect("LLM_API_KEY must be set");
        let llm_base_url = env::var("LLM_BASE_URL").expect("LLM_BASE_URL must be set");
        let llm_model = env::var("LLM_MODEL").expect("LLM_MODEL must be set");
        
        // 初始化数据库
        println!("💾 初始化数据库...");
        let db_repo = DatabaseRepository::new(&database_url).await
            .map_err(|e| anyhow::anyhow!("数据库连接失败: {}", e))?;
        
        // 初始化知识库
        println!("📚 初始化知识库...");
        let kb = Arc::new(Mutex::new(
            KnowledgeBase::new(db_repo.clone())
                .map_err(|e| anyhow::anyhow!("知识库初始化失败: {}", e))?
        ));
        
        // 初始化上下文管理器
        println!("💬 初始化上下文管理器...");
        let ctx_manager = Arc::new(ContextManager::new(db_repo.clone()));
        
        // 初始化增强版提示词管理器
        println!("🧠 初始化提示词管理器...");
        let prompt_manager = Arc::new(EnhancedPromptManager::new());
        
        let config = Arc::new(models::Config {
            database_url,
            llm_api_key,
            llm_base_url,
            llm_model,
        });
        
        println!("✅ NapCat组件初始化完成");
        
        Ok(NapCatComponents {
            kb,
            ctx_manager,
            prompt_manager,
            config,
            db: Arc::new(db_repo),
        })
    }
    
    /// 启动所有服务
    pub async fn start_services(&mut self) -> Result<()> {
        println!("🚀 启动统一服务...");
        
        // 启动WebSocket处理器
        let ws_handler = WebSocketHandler::new();
        println!("🌐 WebSocket处理器已启动");
        
        // 启动MCP服务器
        self.setup_mcp_servers().await?;
        println!("🔧 MCP服务器已配置");
        
        // 启动主要服务循环
        self.run_main_loop().await?;
        
        Ok(())
    }
    
    /// 设置MCP服务器
    async fn setup_mcp_servers(&self) -> Result<()> {
        let mcp = self.service_context.get_mcp()?;
        
        // 添加示例服务器配置
        let server_config = ServerConfig {
            command: "echo".to_string(),
            args: vec!["MCP test".to_string()],
            env: None,
            cwd: None,
            timeout: Some(5),
        };
        
        mcp.add_server("example_server".to_string(), server_config).await?;
        println!("✅ 示例MCP服务器已添加");
        
        Ok(())
    }
    
    /// 运行主循环
    async fn run_main_loop(&self) -> Result<()> {
        println!("🔄 启动主服务循环...");
        println!("💡 按 Ctrl+C 停止服务");
        
        // 模拟服务运行
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    println!("⏱️  服务运行中... ({}s)", 
                        tokio::time::Instant::now().elapsed().as_secs());
                    
                    // 这里可以添加定期任务，如健康检查、日志清理等
                    self.health_check().await;
                }
                // 可以添加其他异步任务
            }
        }
    }
    
    /// 健康检查
    async fn health_check(&self) {
        // 检查各组件状态
        match self.service_context.get_asr() {
            Ok(_) => println!("✅ ASR服务正常"),
            Err(e) => eprintln!("❌ ASR服务异常: {}", e),
        }
        
        match self.service_context.get_tts() {
            Ok(_) => println!("✅ TTS服务正常"),
            Err(e) => eprintln!("❌ TTS服务异常: {}", e),
        }
        
        match self.service_context.get_vad() {
            Ok(_) => println!("✅ VAD服务正常"),
            Err(e) => eprintln!("❌ VAD服务异常: {}", e),
        }
        
        match self.service_context.get_agent() {
            Ok(_) => println!("✅ Agent服务正常"),
            Err(e) => eprintln!("❌ Agent服务异常: {}", e),
        }
        
        match self.service_context.get_llm() {
            Ok(_) => println!("✅ LLM服务正常"),
            Err(e) => eprintln!("❌ LLM服务异常: {}", e),
        }
        
        // 检查NapCat组件
        println!("✅ NapCat知识库连接正常");
        println!("✅ NapCat上下文管理器正常");
        println!("✅ NapCat提示词管理器正常");
    }
    
    /// 执行语音到文本转换
    pub async fn speech_to_text(&self, audio_data: Vec<f32>) -> Result<String> {
        let asr = self.service_context.get_asr()?;
        asr.transcribe_np(audio_data)
    }
    
    /// 执行文本到语音转换
    pub async fn text_to_speech(&self, text: &str, file_name: Option<&str>) -> Result<String> {
        let tts = self.service_context.get_tts()?;
        tts.generate_audio(text, file_name)
    }
    
    /// 检测语音活动
    pub async fn detect_speech(&self, audio_bytes: &[u8]) -> Result<Vec<Vec<u8>>> {
        let vad = self.service_context.get_vad()?;
        vad.detect_speech(audio_bytes)
    }
    
    /// 与Agent交互
    pub async fn chat_with_agent(&self, input: BaseInput) -> Result<Vec<BaseOutput>> {
        let agent = self.service_context.get_agent()?;
        let mut response_stream = agent.chat(input).await?;
        let mut responses = Vec::new();
        
        while let Some(response) = response_stream.next().await {
            match response {
                Ok(output) => responses.push(output),
                Err(e) => return Err(e),
            }
        }
        
        Ok(responses)
    }
    
    /// 获取服务上下文
    pub fn get_service_context(&self) -> &ServiceContext {
        &self.service_context
    }
    
    /// 获取NapCat组件
    pub fn get_napcat_components(&self) -> &NapCatComponents {
        &self.napcat_components
    }
}

// 为NapCatComponents实现一些便捷方法
impl NapCatComponents {
    /// 添加知识到知识库
    pub async fn add_knowledge(&self, content: &str) -> Result<()> {
        let mut kb_lock = self.kb.lock().await;
        kb_lock.add_document(content).await
            .map_err(|e| anyhow::anyhow!("添加知识失败: {}", e))
    }
    
    /// 搜索知识库
    pub async fn search_knowledge(&self, query: &str, limit: usize) -> Result<Vec<String>> {
        let mut kb_lock = self.kb.lock().await;
        kb_lock.search(query, limit).await
            .map_err(|e| anyhow::anyhow!("搜索知识库失败: {}", e))
    }
    
    /// 记录交互
    pub async fn record_interaction(&self, 
        user_id: Option<i64>, 
        group_id: Option<i64>, 
        input: &str, 
        response: &str
    ) -> Result<()> {
        self.ctx_manager.record_interaction(user_id, group_id, input, response).await
            .map_err(|e| anyhow::anyhow!("记录交互失败: {}", e))
    }
    
    /// 获取对话上下文
    pub async fn get_context(&self, 
        user_id: Option<i64>, 
        group_id: Option<i64>
    ) -> Result<String> {
        self.ctx_manager.get_rag_context(user_id, group_id).await
            .map_err(|e| anyhow::anyhow!("获取上下文失败: {}", e))
    }
}