use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State, Path},
    response::IntoResponse,
    routing::{get, post, put, delete},
    Router,
    Json,
};
use futures::{sink::SinkExt, stream::StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{env, sync::Arc};
use tracing::{error, info};
use tower_http::cors::{CorsLayer, Any};

mod db;
mod models;
mod knowledge_base;
mod context_manager;
mod prompts;
mod config;
mod enhanced_prompts;
mod message_parser;
mod image_processor;

use image_processor::{ImageProcessor, ImageServiceConfig};

use napcat_backend::db::DatabaseRepository;
use napcat_backend::knowledge_base::KnowledgeBase;
use napcat_backend::context_manager::ContextManager;
use models::{OneBotEvent, NapCatArrayEvent, ReplyPayload, ReplyParams};
use prompts::{ChatPromptBuilder, LearnPromptBuilder, PromptType};
use crate::config::{AppConfig, ConfigManager as AppConfManager};
use enhanced_prompts::EnhancedPromptManager;

// --- Config ---

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    database_url: String,
    llm_api_key: String,
    llm_base_url: String,
    llm_model: String,
}

impl Config {
    fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            llm_api_key: env::var("LLM_API_KEY").expect("LLM_API_KEY must be set"),
            llm_base_url: env::var("LLM_BASE_URL").expect("LLM_BASE_URL must be set"),
            llm_model: env::var("LLM_MODEL").expect("LLM_MODEL must be set"),
        }
    }
}

// --- App State ---

#[derive(Clone)]
struct AppState {
    kb: Arc<tokio::sync::Mutex<KnowledgeBase>>,
    ctx_manager: Arc<ContextManager>,
    prompt_manager: Arc<EnhancedPromptManager>,
    config: Arc<Config>,
    http_client: Client,
    db: Arc<DatabaseRepository>,
    continuous_learning: Arc<tokio::sync::Mutex<ContinuousLearningState>>,
}

/// 连续学习模式状态
#[derive(Clone, Debug)]
struct ContinuousLearningState {
    is_active: bool,
    start_time: Option<std::time::Instant>,
    collected_messages: Vec<String>,
    user_id: Option<i64>,
    group_id: Option<i64>,
}

impl ContinuousLearningState {
    fn new() -> Self {
        Self {
            is_active: false,
            start_time: None,
            collected_messages: Vec::new(),
            user_id: None,
            group_id: None,
        }
    }
    
    fn start(&mut self, user_id: Option<i64>, group_id: Option<i64>) {
        self.is_active = true;
        self.start_time = Some(std::time::Instant::now());
        self.collected_messages.clear();
        self.user_id = user_id;
        self.group_id = group_id;
        info!("🎯 连续学习模式已启动，用户: {:?}, 群组: {:?}", user_id, group_id);
    }
    
    fn stop(&mut self) {
        self.is_active = false;
        self.start_time = None;
        info!("🛑 连续学习模式已停止，收集了 {} 条消息", self.collected_messages.len());
    }
    
    fn add_message(&mut self, message: String) {
        if self.is_active {
            let preview = message.chars().take(50).collect::<String>();
            self.collected_messages.push(message);
            info!("📥 连续学习模式收集消息: {} (总数: {})", preview, self.collected_messages.len());
        }
    }
    
    fn should_auto_stop(&self) -> bool {
        if let Some(start_time) = self.start_time {
            let elapsed = start_time.elapsed();
            elapsed >= std::time::Duration::from_secs(300) // 5分钟
        } else {
            false
        }
    }
    
    fn get_collected_content(&self) -> String {
        self.collected_messages.join("\n---\n")
    }
}

// --- Main ---

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    dotenvy::dotenv().ok();
    
    // 确保环境变量已加载
    info!("Loading environment variables...");
    
    let config = Arc::new(Config::from_env());

    // 初始化数据库
    info!("Initializing Database...");
    let db_repo = Arc::new(DatabaseRepository::new(&config.database_url).await.unwrap());

    // 初始化知识库
    info!("Initializing Knowledge Base...");
    let kb = Arc::new(tokio::sync::Mutex::new(KnowledgeBase::new((*db_repo).clone()).unwrap()));

    // 初始化上下文管理器
    info!("Initializing Context Manager...");
    let ctx_manager = Arc::new(ContextManager::new((*db_repo).clone()));

    // 初始化配置管理器
    let app_config_manager = match AppConfManager::from_env() {
        Ok(config_manager) => Arc::new(config_manager),
        Err(e) => {
            error!("Failed to initialize config manager: {}", e);
            // 使用默认配置管理器
            Arc::new(AppConfManager::new())
        }
    };
    
    // 初始化增强版提示词管理器
    let prompt_manager = Arc::new(EnhancedPromptManager::from_config((*app_config_manager).clone()));
    
    info!("提示词系统初始化完成");
    info!("可用提示词类型: {:?}", prompt_manager.get_base_manager().get_available_types());

    let state = AppState {
        kb,
        ctx_manager,
        prompt_manager,
        config,
        http_client: Client::new(),
        db: db_repo,
        continuous_learning: Arc::new(tokio::sync::Mutex::new(ContinuousLearningState::new())),
    };

    // 启动连续学习模式检查任务
    let state_clone = state.clone();
    tokio::spawn(async move {
        continuous_learning_checker(state_clone).await;
    });

    // 3. Start WebSocket Server to listen for NapCat connections
    let napcat_token = env::var("NAPCAT_TOKEN").expect("NAPCAT_TOKEN must be set");
    let listen_addr = "0.0.0.0:8082";
    
    info!("Starting WebSocket server on {} for NapCat connections...", listen_addr);
    
    // 允许 Flutter 跨域访问
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Create Axum app with WebSocket route - support both root and /ws paths
    let app = Router::new()
        .route("/", get(ws_handler))  // 根路径，用于NapCat反向WebSocket
        .route("/ws", get(ws_handler))  // /ws路径，备用
        // === 新增管理接口 ===
        .route("/api/config", get(get_config).post(update_config))
        .route("/api/knowledge/search", get(search_knowledge_get))  // 明确的搜索路由
        .route("/api/knowledge", get(search_knowledge_get).post(add_knowledge))  // 兼容旧路由
        .route("/api/knowledge/list", get(list_knowledge))
        .route("/api/knowledge/:id", get(get_knowledge_by_id).put(update_knowledge).delete(delete_knowledge))
        .route("/api/prompts", get(get_prompts).post(update_prompts))
        .route("/api/prompts/stats", get(get_prompts_stats))
        .route("/api/system/info", get(get_system_info))
        .layer(cors) // 应用 CORS
        .with_state(state);

    // Start the server
    let listener = tokio::net::TcpListener::bind(listen_addr).await.unwrap();
    info!("✅ WebSocket server listening on {}", listen_addr);
    info!("✅ Management API available at http://{}/api/*", listen_addr);
    
    axum::serve(listener, app).await.unwrap();
}


async fn connect_to_napcat(url: &str, _token: &str, state: AppState) {
    loop {
        match tokio_tungstenite::connect_async(url).await {
            Ok((ws_stream, _)) => {
                info!("✅ Successfully connected to NapCat server");
                handle_napcat_connection(ws_stream, state.clone()).await;
            }
            Err(e) => {
                error!("❌ Failed to connect to NapCat server: {}. Retrying in 5 seconds...", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}

async fn handle_napcat_connection(ws_stream: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, state: AppState) {
    use futures::{SinkExt, StreamExt};
    use tokio::sync::mpsc;
    
    let (write, mut read) = ws_stream.split();
    let (tx, mut rx) = mpsc::channel::<String>(100);
    
    // Spawn writer task
    let mut write_task = write;
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = write_task.send(tokio_tungstenite::tungstenite::Message::Text(msg)).await {
                error!("Failed to send message to NapCat: {}", e);
                break;
            }
        }
    });
    
    // Send authentication if needed
    // NapCat may require specific authentication format
    
    while let Some(msg) = read.next().await {
        match msg {
            Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                info!("📨 Received from NapCat: {}", text);
                
                // Try to parse as Array format first (NapCat format)
                if let Ok(array_event) = serde_json::from_str::<NapCatArrayEvent>(&text) {
                    if array_event.0 == "message" {
                        // Convert Array format to OneBotEvent
                        if let Ok(event_data) = serde_json::from_value::<OneBotEvent>(array_event.1) {
                            let state_clone = state.clone();
                            let tx_clone = tx.clone();
                            
                            tokio::spawn(async move {
                                let reply = process_napcat_message(&event_data, &state_clone).await;
                                if let Some(reply_msg) = reply {
                                    // reply_msg is already a WebSocket Message, extract the text
                                    if let tokio_tungstenite::tungstenite::Message::Text(text) = reply_msg {
                                        let _ = tx_clone.send(text).await;
                                    }
                                }
                            });
                        }
                    }
                } else if let Ok(event) = serde_json::from_str::<OneBotEvent>(&text) {
                    // Handle standard JSON format
                    if event.post_type.as_deref() == Some("message") {
                        let state_clone = state.clone();
                        let tx_clone = tx.clone();
                        
                        tokio::spawn(async move {
                            let reply = process_napcat_message(&event, &state_clone).await;
                            if let Some(reply_msg) = reply {
                                // reply_msg is already a WebSocket Message, extract the text
                                if let tokio_tungstenite::tungstenite::Message::Text(text) = reply_msg {
                                    let _ = tx_clone.send(text).await;
                                }
                            }
                        });
                    }
                }
            }
            Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => {
                info!("🔒 NapCat connection closed");
                break;
            }
            Err(e) => {
                error!("❌ WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
    
    info!("🔌 Disconnected from NapCat server");
}

async fn process_napcat_message(event: &OneBotEvent, state: &AppState) -> Option<tokio_tungstenite::tungstenite::Message> {
    // Similar to the original process_event function, but returns WebSocket message
    let raw_msg = event.raw_message.clone().unwrap_or_default();
    
    if raw_msg.trim().is_empty() {
        return None;
    }
    
    info!("📝 Processing message: {}", raw_msg);

    // 群聊消息检查是否被@
    if let Some(_group_id) = event.group_id {
        // 机器人QQ号，从环境变量获取或硬编码
        let bot_qq = env::var("BOT_QQ").unwrap_or_else(|_| "3955516984".to_string());
        let at_pattern = format!("[CQ:at,qq={}]", bot_qq);
        
        // 检查是否被@，如果没有被@则忽略消息
        if !raw_msg.contains(&at_pattern) {
            info!("⏭️  群聊消息未@机器人，忽略消息");
            return None;
        }
        info!("✅ 检测到@机器人消息");
    }
    
    // --- Command: Learn ---
    if raw_msg.starts_with("/learn ") {
        let content = raw_msg.replace("/learn ", "");
        let mut kb_lock = state.kb.lock().await;
        if let Err(e) = kb_lock.add_document(&content).await {
            error!("Learn failed: {}", e);
            return Some(create_napcat_reply(event, "Failed to learn."));
        } else {
            return Some(create_napcat_reply(event, "Knowledge stored successfully."));
        }
    }
    
    // --- Standard Chat ---
    info!("原始消息: {}", raw_msg);
    
    // 首先清理CQ码，避免AI被干扰
    let clean_msg = message_parser::remove_cq_codes(&raw_msg);
    info!("清理后的消息: {}", clean_msg);


    // 解析消息内容，包括转发消息
    let parsed_message = message_parser::parse_message(event);
    let searchable_content = message_parser::get_searchable_content(&parsed_message);
    
    info!("解析后的搜索内容: {}", searchable_content);
    if parsed_message.has_forward {
        info!("检测到转发消息，内容长度: {}", parsed_message.forward_content.as_ref().map(|s| s.len()).unwrap_or(0));
    }
    if parsed_message.has_reply {
        info!("检测到回复消息");
    }

    // 0. Auto-learn from forwarded messages if enabled
    if let Some(config_manager) = state.prompt_manager.get_config_manager() {
        let config = config_manager.get_config();
        if config.system.auto_learn_enabled && parsed_message.has_forward {
            if let Some(forward_content) = &parsed_message.forward_content {
                let content_length = forward_content.len();
                if content_length >= config.system.auto_learn_min_length && 
                   content_length <= config.system.auto_learn_max_length {
                    
                    // 清理内容格式（如果不需要学习格式）
                    let content_to_learn = if !config.system.learn_message_format {
                        // 移除用户名和时间戳格式，只保留纯内容
                        clean_message_content(forward_content)
                    } else {
                        forward_content.clone()
                    };
                    
                    let mut kb_lock = state.kb.lock().await;
                    match kb_lock.add_document(&content_to_learn).await {
                        Ok(_) => {
                            info!("✅ 自动学习成功：从转发消息中提取了 {} 字符的内容", content_to_learn.len());
                        }
                        Err(e) => {
                            error!("❌ 自动学习失败：{}", e);
                        }
                    }
                } else {
                    info!("⏭️  转发内容长度 {} 不在学习范围内（{}-{}）", 
                          content_length, config.system.auto_learn_min_length, config.system.auto_learn_max_length);
                }
            }
        }
    }

    // 1. Gather Knowledge (RAG)
    let mut kb_lock = state.kb.lock().await;
    let knowledge_docs = kb_lock.search(&searchable_content, 10).await.unwrap_or_default();
    
    // 如果没有知识库结果但有转发内容，将转发内容作为临时知识
    let knowledge_str = if knowledge_docs.is_empty() && parsed_message.has_forward {
        if let Some(forward_content) = &parsed_message.forward_content {
            info!("📚 使用转发内容作为临时知识库");
            format!("转发消息内容：\n{}", forward_content)
        } else {
            String::new()
        }
    } else {
        knowledge_docs.join("\n---\n")
    };
    
    // 2. Gather Conversation Context
    let history_str = state.ctx_manager.get_rag_context(event.user_id, event.group_id).await.unwrap_or_default();
    
    // 3. Build Prompt using enhanced Prompt System
    let (system_prompt, user_prompt) = match state.prompt_manager.build_smart_prompt(
        raw_msg.clone(),
        Some(knowledge_str),
        Some(history_str),
    ) {
        Ok((system, user)) => (system, user),
        Err(e) => {
            error!("Failed to build prompt: {}", e);
            return Some(create_napcat_reply(event, "抱歉，构建提示词时出错。"));
        }
    };

    // 4. Call LLM
    match call_llm(&state, &system_prompt, &user_prompt).await {
        Ok(response) => {
            let _ = state.ctx_manager.record_interaction(event.user_id, event.group_id, &raw_msg, &response).await;
            Some(create_napcat_reply(event, &response))
        }
        Err(e) => {
            error!("LLM Error: {}", e);
            Some(create_napcat_reply(event, "我现在有些困扰，请稍后再试。"))
        }
    }
}

fn create_napcat_reply(event: &OneBotEvent, msg: &str) -> tokio_tungstenite::tungstenite::Message {
    let reply = ReplyPayload {
        action: "send_msg".to_string(),
        params: ReplyParams {
            user_id: event.user_id,
            group_id: event.group_id,
            message: msg.to_string(),
        },
    };
    
    let json_str = serde_json::to_string(&reply).unwrap();
    tokio_tungstenite::tungstenite::Message::Text(json_str)
}

/// 清理消息内容，移除用户名和时间戳格式
fn clean_message_content(content: &str) -> String {
    let mut cleaned = content.to_string();
    
    // 移除常见的用户名格式，如 "用户名: " 或 "用户名："
    cleaned = regex::Regex::new(r"^[^:：]+[:：]\s*").unwrap().replace_all(&cleaned, "").to_string();
    
    // 移除时间戳格式，如 "[HH:MM]" 或 "[HH:MM:SS]"
    cleaned = regex::Regex::new(r"\[\d{1,2}:\d{2}(?::\d{2})?\]\s*").unwrap().replace_all(&cleaned, "").to_string();
    
    // 移除日期格式，如 "[YYYY-MM-DD]" 或 "[YYYY/MM/DD]"
    cleaned = regex::Regex::new(r"\[\d{4}[-/]\d{1,2}[-/]\d{1,2}\]\s*").unwrap().replace_all(&cleaned, "").to_string();
    
    // 移除多余的空白字符
    cleaned = cleaned.split_whitespace().collect::<Vec<&str>>().join(" ");
    
    cleaned.trim().to_string()
}

/// 连续学习模式后台检查任务
async fn continuous_learning_checker(state: AppState) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30)); // 每30秒检查一次
    
    loop {
        interval.tick().await;
        
        let mut learning_state = state.continuous_learning.lock().await;
        if learning_state.is_active && learning_state.should_auto_stop() {
            info!("⏰ 连续学习模式达到5分钟，自动停止");
            
            // 收集的内容写入知识库
            let collected_content = learning_state.get_collected_content();
            if !collected_content.trim().is_empty() {
                let mut kb_lock = state.kb.lock().await;
                match kb_lock.add_document(&collected_content).await {
                    Ok(_) => {
                        info!("✅ 连续学习模式：成功将 {} 字符的内容写入知识库", collected_content.len());
                    }
                    Err(e) => {
                        error!("❌ 连续学习模式：写入知识库失败: {}", e);
                    }
                }
            }
            
            learning_state.stop();
        }
    }
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);

    info!("🔄 新的WebSocket连接已建立");

    // Writer Task
    tokio::spawn(async move {
        loop {
            if let Some(msg) = rx.recv().await {
                if let Err(e) = sender.send(Message::Text(msg)).await {
                    error!("WS Send Error: {}", e);
                    break;
                }
            }
        }
    });

    // Reader Loop
    while let Some(msg_result) = receiver.next().await {
        match msg_result {
            Ok(Message::Text(text)) => {
                info!("📨 WebSocket收到文本消息: {}", text);
                let state = state.clone();
                let tx = tx.clone();
                tokio::spawn(async move {
                    process_event(&text, state, tx).await;
                });
            }
            Ok(Message::Binary(data)) => {
                info!("📦 WebSocket收到二进制消息: {} bytes", data.len());
                info!("📊 二进制消息内容: {:?}", data);
            }
            Ok(Message::Ping(data)) => {
                info!("🏓 WebSocket Ping: {:?}", data);
            }
            Ok(Message::Pong(data)) => {
                info!("🏸 WebSocket Pong: {:?}", data);
            }
            Ok(Message::Close(frame)) => {
                info!("🔒 WebSocket关闭: {:?}", frame);
                break;
            }
            Err(e) => {
                error!("❌ WebSocket错误: {}", e);
                break;
            }
        }
    }
    
    info!("🔌 WebSocket连接已断开");
}

async fn process_event(json_str: &str, state: AppState, tx: tokio::sync::mpsc::Sender<String>) {
    info!("📨 收到事件: {}", json_str);
    
    let event: OneBotEvent = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            info!("❌ 事件解析失败: {}", e);
            return;
        }
    };

    info!("📋 事件类型: {:?}", event.post_type);
    info!("👤 用户ID: {:?}", event.user_id);
    info!("👥 群组ID: {:?}", event.group_id);
    info!("💬 原始消息: {:?}", event.raw_message);

    if event.post_type.as_deref() != Some("message") {
        info!("⏭️  忽略非消息事件");
        return;
    }

    // 验证必要字段
    if event.user_id.is_none() {
        info!("⚠️  缺少用户ID，忽略事件");
        return;
    }

    let raw_msg = event.raw_message.clone().unwrap_or_default();
    
    if raw_msg.trim().is_empty() {
        info!("⚠️  空消息内容，忽略事件");
        return;
    }

    info!("📝 处理消息: {}", raw_msg);

    // 群聊消息检查是否被@
    if let Some(group_id) = event.group_id {
        // 机器人QQ号，从环境变量获取或硬编码
        let bot_qq = env::var("BOT_QQ").unwrap_or_else(|_| "3955516984".to_string());
        let at_pattern = format!("[CQ:at,qq={}]", bot_qq);
        
        // 检查是否被@，如果没有被@则忽略消息
        if !raw_msg.contains(&at_pattern) {
            info!("⏭️  群聊消息未@机器人，忽略消息");
            return;
        }
        info!("✅ 检测到@机器人消息");
    }


    // --- Command: Learn ---
    if raw_msg.starts_with("/learn ") {
        info!("📝 检测到/learn命令，原始消息: {}", raw_msg);
        let content = raw_msg.replace("/learn ", "");
        info!("📋 提取的学习内容: '{}'", content);
        
        if content.trim().is_empty() {
            info!("⚠️ 学习内容为空，跳过学习");
            send_msg(&tx, &event, "学习内容为空，请输入要学习的文本。").await;
            return;
        }
        
        let mut kb_lock = state.kb.lock().await;
        info!("🚀 开始将内容添加到知识库，长度: {} 字符", content.len());
        
        match kb_lock.add_document(&content).await {
            Ok(_) => {
                info!("✅ 学习成功：已将 {} 字符的内容添加到知识库", content.len());
                send_msg(&tx, &event, &format!("学习成功！已将 {} 字符的内容添加到知识库。", content.len())).await;
            }
            Err(e) => {
                error!("❌ 学习失败: {}", e);
                send_msg(&tx, &event, &format!("学习失败: {}", e)).await;
            }
        }
        return;
    }

    // --- Command: Continuous Learning Mode ---
    if raw_msg.starts_with("/连续学习") || raw_msg.starts_with("/continuous_learn") {
        let mut learning_state = state.continuous_learning.lock().await;
        if learning_state.is_active {
            // 如果已经在运行，则停止
            let collected_content = learning_state.get_collected_content();
            if !collected_content.trim().is_empty() {
                let mut kb_lock = state.kb.lock().await;
                match kb_lock.add_document(&collected_content).await {
                    Ok(_) => {
                        info!("✅ 连续学习模式：手动停止，成功将 {} 字符的内容写入知识库", collected_content.len());
                        send_msg(&tx, &event, &format!("连续学习模式已停止，成功学习了 {} 条消息，共 {} 字符", 
                            learning_state.collected_messages.len(), collected_content.len())).await;
                    }
                    Err(e) => {
                        error!("❌ 连续学习模式：写入知识库失败: {}", e);
                        send_msg(&tx, &event, "连续学习模式停止，但写入知识库失败。").await;
                    }
                }
            } else {
                send_msg(&tx, &event, "连续学习模式已停止，但没有收集到任何内容。").await;
            }
            learning_state.stop();
        } else {
            // 启动连续学习模式
            learning_state.start(event.user_id, event.group_id);
            send_msg(&tx, &event, "🎯 连续学习模式已启动！接下来的5分钟内，我会自动收集所有消息并学习。发送 /连续学习 或 /continuous_learn 可以手动停止。").await;
        }
        return;
    }

    // --- Database Query Commands ---
    
    // Command: 查看数据库统计信息
    if raw_msg == "/db_stats" || raw_msg == "/数据库统计" {
        let stats = get_database_stats(&state).await;
        send_msg(&tx, &event, &stats).await;
        return;
    }

    // Command: 查看最近的对话记录
    if raw_msg.starts_with("/recent ") || raw_msg.starts_with("/最近对话 ") {
        let parts: Vec<&str> = raw_msg.split_whitespace().collect();
        let limit = if parts.len() > 1 {
            parts[1].parse::<i64>().unwrap_or(5)
        } else {
            5
        };
        
        let recent_conversations = get_recent_conversations_summary(&state, limit).await;
        send_msg(&tx, &event, &recent_conversations).await;
        return;
    }

    // Command: 查看知识库文档
    if raw_msg == "/docs" || raw_msg == "/知识库" {
        let docs_list = get_documents_list(&state).await;
        send_msg(&tx, &event, &docs_list).await;
        return;
    }

    // Command: 查看系统配置
    if raw_msg == "/config" || raw_msg == "/配置" {
        let config_info = get_system_config(&state).await;
        send_msg(&tx, &event, &config_info).await;
        return;
    }

    // Command: 查看用户上下文
    if raw_msg == "/user_ctx" || raw_msg == "/用户上下文" {
        if let Some(user_id) = event.user_id {
            let user_context = get_user_context(&state, user_id).await;
            send_msg(&tx, &event, &user_context).await;
        } else {
            send_msg(&tx, &event, "无法获取用户ID，无法查看用户上下文。").await;
        }
        return;
    }

    // Command: 查看群组上下文
    if raw_msg == "/group_ctx" || raw_msg == "/群组上下文" {
        if let Some(group_id) = event.group_id {
            let group_context = get_group_context(&state, group_id).await;
            send_msg(&tx, &event, &group_context).await;
        } else {
            send_msg(&tx, &event, "无法获取群组ID，无法查看群组上下文。").await;
        }
        return;
    }

    // 首先清理CQ码，避免AI被干扰
    let clean_msg = message_parser::remove_cq_codes(&raw_msg);
    info!("清理后的消息: {}", clean_msg);


    // 解析消息内容，包括转发消息
    let parsed_message = message_parser::parse_message(&event);
    let searchable_content = message_parser::get_searchable_content(&parsed_message);
    
    // 检查连续学习模式是否激活，如果是则收集消息
    {
        let mut learning_state = state.continuous_learning.lock().await;
        if learning_state.is_active {
            // 清理消息内容（移除@机器人的部分）
            let clean_content = raw_msg.replace(&format!("[CQ:at,qq={}]", env::var("BOT_QQ").unwrap_or_else(|_| "3955516984".to_string())), "").trim().to_string();
            if !clean_content.is_empty() {
                learning_state.add_message(clean_content);
            }
        }
    }

    // --- 引用内容自动学习 ---
    if parsed_message.has_reply || parsed_message.has_forward {
        if let Some(config_manager) = state.prompt_manager.get_config_manager() {
            let config = config_manager.get_config();
            if config.system.auto_learn_enabled {
                if let Some(forward_content) = &parsed_message.forward_content {
                    info!("📝 检测到引用/转发消息，自动学习内容（长度: {}）", forward_content.len());
                    let content_to_learn = if !config.system.learn_message_format {
                        clean_message_content(forward_content)
                    } else {
                        forward_content.clone()
                    };
                    
                    if !content_to_learn.is_empty() {
                        let mut kb_lock = state.kb.lock().await;
                        match kb_lock.add_document(&content_to_learn).await {
                            Ok(_) => {
                                info!("✅ 引用内容自动学习成功：{} 字符", content_to_learn.len());
                            }
                            Err(e) => {
                                error!("❌ 引用内容自动学习失败：{}", e);
                            }
                        }
                    }
                }
            }
        }
    }

    // --- Standard Chat ---
    info!("Query: {}", raw_msg);
    
    info!("解析后的搜索内容: {}", searchable_content);
    if parsed_message.has_forward {
        info!("检测到转发消息，内容长度: {}", parsed_message.forward_content.as_ref().map(|s| s.len()).unwrap_or(0));
    }
    if parsed_message.has_reply {
        info!("检测到回复消息");
    }

    // 1. Build Context with Priority: Forward Content > Knowledge Base > History
    let mut context_parts = Vec::new();
    
    // 优先加入转发内容（最重要的当前上下文）
    if let Some(forward_content) = &parsed_message.forward_content {
        if !forward_content.trim().is_empty() {
            context_parts.push(format!("=== 当前参考的聊天记录/上下文 ===\n{}\n==============================\n", forward_content));
            info!("✅ 已将转发内容加入提示词上下文，长度: {}", forward_content.len());
        }
    }
    
    // 2. Gather Knowledge (RAG)
    let mut kb_lock = state.kb.lock().await;
    let knowledge_docs = kb_lock.search(&searchable_content, 3).await.unwrap_or_default();
    if !knowledge_docs.is_empty() {
        let knowledge_str = knowledge_docs.join("\n---\n");
        context_parts.push(format!("=== 知识库参考资料 ===\n{}\n\n", knowledge_str));
        info!("知识库搜索结果: {} 个文档", knowledge_docs.len());
        info!("知识库内容预览: {}", &knowledge_str.chars().take(200).collect::<String>());
    } else {
        info!("知识库搜索结果: 0 个文档");
    }

    // 3. Gather Conversation Context
    let history_str = state.ctx_manager.get_rag_context(event.user_id, event.group_id).await.unwrap_or_default();
    if !history_str.is_empty() {
        context_parts.push(format!("=== 对话历史 ===\n{}\n\n", history_str));
        info!("对话历史: 有内容 (长度: {})", history_str.len());
    } else {
        info!("对话历史: 空");
    }
    
    // 组合所有上下文
    let full_context = context_parts.join("");
    info!("构建提示词，总上下文长度: {}", full_context.len());

    // 4. Build Prompt using enhanced Prompt System
    let (system_prompt, user_prompt) = match state.prompt_manager.build_smart_prompt(
        clean_msg.clone(), // 使用清理后的消息，避免CQ码干扰
        Some(full_context),
        None, // 历史已经包含在context中
    ) {
        Ok((system, user)) => {
            info!("提示词构建成功，系统提示长度: {}, 用户提示长度: {}", system.len(), user.len());
            (system, user)
        },
        Err(e) => {
            error!("Failed to build prompt: {}", e);
            send_msg(&tx, &event, "抱歉，构建提示词时出错。").await;
            return;
        }
    };

    // 4. Call LLM
    match call_llm(&state, &system_prompt, &user_prompt).await {
        Ok(response) => {
            let _ = state.ctx_manager.record_interaction(event.user_id, event.group_id, &raw_msg, &response).await;
            send_msg(&tx, &event, &response).await;
        }
        Err(e) => {
            error!("LLM Error: {}", e);
            send_msg(&tx, &event, "我现在有些困扰，请稍后再试。").await;
        }
    }
}

async fn send_msg(tx: &tokio::sync::mpsc::Sender<String>, event: &OneBotEvent, msg: &str) {
    let payload = ReplyPayload {
        action: "send_msg".to_string(),
        params: ReplyParams {
            user_id: event.user_id,
            group_id: event.group_id,
            message: msg.to_string(),
        },
    };
    if let Ok(s) = serde_json::to_string(&payload) {
        let _ = tx.send(s).await;
    }
}

// === 管理API处理函数 ===

// 获取当前配置（不包含敏感信息）
async fn get_config(State(state): State<AppState>) -> impl IntoResponse {
    info!("获取系统配置");
    
    Json(serde_json::json!({
        "status": "success",
        "data": {
            "llm_base_url": state.config.llm_base_url,
            "llm_model": state.config.llm_model,
            "prompt_stats": state.prompt_manager.get_stats(),
            "system_info": {
                "knowledge_search_limit": state.prompt_manager.get_knowledge_search_limit(),
                "similarity_threshold": state.prompt_manager.get_similarity_threshold(),
                "max_history_length": state.prompt_manager.get_max_history_length(),
                "log_level": state.prompt_manager.get_log_level(),
            }
        }
    }))
}

// 更新配置（模拟）
async fn update_config(State(_state): State<AppState>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    info!("收到配置更新请求: {:?}", payload);
    
    // 这里应该将 payload 写入 .env 文件或数据库中的 system_config 表
    // 为了安全起见，不直接更新运行时配置，而是提示需要重启
    
    Json(serde_json::json!({
        "status": "success", 
        "message": "配置更新已接收，需要重启服务以应用更改",
        "note": "在实际生产环境中，应该将配置保存到数据库或配置文件"
    }))
}

// 搜索知识库 - GET请求处理（通过查询参数）
async fn search_knowledge_get(State(state): State<AppState>, axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>) -> impl IntoResponse {
    info!("搜索知识库 (GET): {:?}", params);
    
    let query = params.get("query").map(|s| s.as_str()).unwrap_or("");
    let limit = params.get("limit").and_then(|l| l.parse::<usize>().ok()).unwrap_or(5);
    
    if query.is_empty() {
        return Json(serde_json::json!({
            "status": "error",
            "message": "查询内容不能为空"
        }));
    }
    
    info!("开始搜索知识库，查询: {}, 限制: {}", query, limit);
    let search_limit = state.prompt_manager.get_knowledge_search_limit();
    let mut kb_lock = state.kb.lock().await;
    
    match kb_lock.search(query, search_limit.min(limit)).await {
        Ok(results) => {
            info!("搜索成功，找到 {} 个结果", results.len());
            Json(serde_json::json!({
                "status": "success",
                "data": {
                    "query": query,
                    "results": results,
                    "count": results.len()
                }
            }))
        }
        Err(e) => {
            error!("知识库搜索失败: {}", e);
            Json(serde_json::json!({
                "status": "error",
                "message": format!("搜索失败: {}", e)
            }))
        }
    }
}

// 搜索知识库 - POST请求处理（通过JSON体）
async fn search_knowledge(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    info!("搜索知识库 (POST): {:?}", payload);
    
    let query = payload.get("query").and_then(|q| q.as_str()).unwrap_or("");
    let limit = payload.get("limit").and_then(|l| l.as_u64()).unwrap_or(5) as usize;
    
    if query.is_empty() {
        return Json(serde_json::json!({
            "status": "error",
            "message": "查询内容不能为空"
        }));
    }
    
    let search_limit = state.prompt_manager.get_knowledge_search_limit();
    let mut kb_lock = state.kb.lock().await;
    
    match kb_lock.search(query, search_limit.min(limit)).await {
        Ok(results) => {
            Json(serde_json::json!({
                "status": "success",
                "data": {
                    "query": query,
                    "results": results,
                    "count": results.len()
                }
            }))
        }
        Err(e) => {
            error!("知识库搜索失败: {}", e);
            Json(serde_json::json!({
                "status": "error",
                "message": format!("搜索失败: {}", e)
            }))
        }
    }
}

// 添加知识
async fn add_knowledge(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    info!("添加知识: {:?}", payload);
    
    let content = payload.get("content").and_then(|c| c.as_str()).unwrap_or("");
    
    if content.is_empty() {
        return Json(serde_json::json!({
            "status": "error",
            "message": "知识内容不能为空"
        }));
    }
    
    let mut kb_lock = state.kb.lock().await;
    match kb_lock.add_document(content).await {
        Ok(_) => {
            Json(serde_json::json!({
                "status": "success",
                "message": "知识添加成功"
            }))
        }
        Err(e) => {
            error!("知识添加失败: {}", e);
            Json(serde_json::json!({
                "status": "error",
                "message": format!("添加失败: {}", e)
            }))
        }
    }
}

// 获取提示词信息
async fn get_prompts(State(state): State<AppState>) -> impl IntoResponse {
    info!("获取提示词信息");
    
    let available_types: Vec<String> = state.prompt_manager.get_base_manager()
        .get_available_types()
        .iter()
        .map(|t| format!("{:?}", t))
        .collect();
    
    Json(serde_json::json!({
        "status": "success",
        "data": {
            "available_types": available_types,
            "stats": state.prompt_manager.get_stats(),
            "smart_selection_enabled": true
        }
    }))
}

// 更新提示词配置
async fn update_prompts(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    info!("更新提示词配置: {:?}", payload);
    
    // 这里可以实现提示词模板的动态更新
    // 为了演示，返回成功信息
    Json(serde_json::json!({
        "status": "success",
        "message": "提示词配置更新成功",
        "note": "在实际生产环境中，应该实现具体的提示词更新逻辑"
    }))
}

// 获取提示词统计信息
async fn get_prompts_stats(State(state): State<AppState>) -> impl IntoResponse {
    info!("获取提示词统计信息");
    
    Json(serde_json::json!({
        "status": "success",
        "data": state.prompt_manager.get_stats()
    }))
}

// 获取系统信息
async fn get_system_info(State(state): State<AppState>) -> impl IntoResponse {
    info!("获取系统信息");
    
    Json(serde_json::json!({
        "status": "success",
        "data": {
            "system": {
                "knowledge_search_limit": state.prompt_manager.get_knowledge_search_limit(),
                "similarity_threshold": state.prompt_manager.get_similarity_threshold(),
                "max_history_length": state.prompt_manager.get_max_history_length(),
                "log_level": state.prompt_manager.get_log_level(),
            },
            "llm": {
                "base_url": state.config.llm_base_url,
                "model": state.config.llm_model,
            },
            "features": {
                "smart_prompt_selection": true,
                "configurable_prompts": true,
                "knowledge_base": true,
                "conversation_context": true,
            }
        }
    }))
}

// 获取知识库列表
async fn list_knowledge(State(state): State<AppState>) -> impl IntoResponse {
    info!("获取知识库列表");
    
    match state.db.list_documents().await {
        Ok(documents) => {
            Json(serde_json::json!({
                "status": "success",
                "data": {
                    "documents": documents,
                    "count": documents.len()
                }
            }))
        }
        Err(e) => {
            error!("获取知识库列表失败: {}", e);
            Json(serde_json::json!({
                "status": "error",
                "message": format!("获取列表失败: {}", e)
            }))
        }
    }
}

// 根据ID获取知识库内容
async fn get_knowledge_by_id(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    info!("获取知识库内容: {}", id);
    
    match state.db.get_document_by_id(&id).await {
        Ok(Some(document)) => {
            Json(serde_json::json!({
                "status": "success",
                "data": document
            }))
        }
        Ok(None) => {
            Json(serde_json::json!({
                "status": "error",
                "message": "文档未找到"
            }))
        }
        Err(e) => {
            error!("获取文档失败: {}", e);
            Json(serde_json::json!({
                "status": "error",
                "message": format!("获取文档失败: {}", e)
            }))
        }
    }
}

// 更新知识库内容
async fn update_knowledge(State(state): State<AppState>, Path(id): Path<String>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    info!("更新知识库内容: {}", id);
    
    let content = payload.get("content").and_then(|c| c.as_str()).unwrap_or("");
    
    if content.is_empty() {
        return Json(serde_json::json!({
            "status": "error",
            "message": "知识内容不能为空"
        }));
    }
    
    match state.db.update_document(&id, content).await {
        Ok(_) => {
            Json(serde_json::json!({
                "status": "success",
                "message": "知识更新成功"
            }))
        }
        Err(e) => {
            error!("知识更新失败: {}", e);
            Json(serde_json::json!({
                "status": "error",
                "message": format!("更新失败: {}", e)
            }))
        }
    }
}

// 删除知识库内容
async fn delete_knowledge(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    info!("删除知识库内容: {}", id);
    
    match state.db.delete_document(&id).await {
        Ok(_) => {
            Json(serde_json::json!({
                "status": "success",
                "message": "知识删除成功"
            }))
        }
        Err(e) => {
            error!("知识删除失败: {}", e);
            Json(serde_json::json!({
                "status": "error",
                "message": format!("删除失败: {}", e)
            }))
        }
    }
}

// --- Database Query Helper Functions ---

async fn get_database_stats(state: &AppState) -> String {
    match state.db.get_conversations_count().await {
        Ok(conv_count) => {
            match state.db.get_documents_count().await {
                Ok(doc_count) => {
                    match state.db.get_user_contexts_count().await {
                        Ok(user_ctx_count) => {
                            match state.db.get_group_contexts_count().await {
                                Ok(group_ctx_count) => {
                                    format!(
                                        "📊 数据库统计信息:\n\n\
                                        💬 对话记录数量: {}\n\
                                        📚 知识库文档数量: {}\n\
                                        👤 用户上下文数量: {}\n\
                                        👥 群组上下文数量: {}",
                                        conv_count, doc_count, user_ctx_count, group_ctx_count
                                    )
                                }
                                Err(e) => format!("❌ 获取群组上下文数量失败: {}", e)
                            }
                        }
                        Err(e) => format!("❌ 获取用户上下文数量失败: {}", e)
                    }
                }
                Err(e) => format!("❌ 获取知识库文档数量失败: {}", e)
            }
        }
        Err(e) => format!("❌ 获取对话记录数量失败: {}", e)
    }
}

async fn get_recent_conversations_summary(state: &AppState, limit: i64) -> String {
    match state.db.get_recent_conversations_summary(limit).await {
        Ok(conversations) => {
            if conversations.is_empty() {
                return "📭 没有找到最近的对话记录。".to_string();
            }
            
            let mut result = format!("💬 最近 {} 条对话记录:\n\n", conversations.len());
            
            for (i, conv) in conversations.iter().enumerate() {
                let id = conv["id"].as_str().unwrap_or("N/A");
                let user_id = conv["user_id"].as_i64();
                let group_id = conv["group_id"].as_i64();
                let raw_message = conv["raw_message"].as_str().unwrap_or("N/A");
                let bot_response = conv["bot_response"].as_str().unwrap_or("N/A");
                let created_at = conv["created_at"].as_str().unwrap_or("N/A");
                
                result.push_str(&format!(
                    "{}. [{}] ID: {}\n",
                    i + 1,
                    created_at.split('.').next().unwrap_or("N/A"), // 只显示时间部分
                    id
                ));
                
                if let Some(uid) = user_id {
                    result.push_str(&format!("   用户: {}\n", uid));
                }
                
                if let Some(gid) = group_id {
                    result.push_str(&format!("   群组: {}\n", gid));
                }
                
                result.push_str(&format!("   消息: {}\n", raw_message));
                if !bot_response.is_empty() && bot_response != "N/A" {
                    result.push_str(&format!("   回复: {}\n", bot_response));
                }
                
                result.push('\n');
            }
            
            result
        }
        Err(e) => format!("❌ 获取最近对话记录失败: {}", e)
    }
}

async fn get_documents_list(state: &AppState) -> String {
    match state.db.list_documents().await {
        Ok(documents) => {
            if documents.is_empty() {
                return "📭 知识库中没有文档。".to_string();
            }
            
            let mut result = format!("📚 知识库中的 {} 个文档:\n\n", documents.len());
            
            for (i, doc) in documents.iter().take(10).enumerate() { // 只显示前10个
                let id = doc["id"].as_str().unwrap_or("N/A");
                let content = doc["content"].as_str().unwrap_or("N/A");
                let created_at = doc["created_at"].as_str().unwrap_or("N/A");
                
                result.push_str(&format!(
                    "{}. [{}] {}\n   内容预览: {}...\n\n",
                    i + 1,
                    created_at.split('.').next().unwrap_or("N/A"),
                    id,
                    content.chars().take(100).collect::<String>()
                ));
            }
            
            if documents.len() > 10 {
                result.push_str(&format!("... 还有 {} 个文档未显示", documents.len() - 10));
            }
            
            result
        }
        Err(e) => format!("❌ 获取知识库文档列表失败: {}", e)
    }
}

async fn get_system_config(state: &AppState) -> String {
    match state.db.get_system_config().await {
        Ok(configs) => {
            if configs.is_empty() {
                return "⚙️ 系统配置表为空。".to_string();
            }
            
            let mut result = "⚙️ 系统配置信息:\n\n".to_string();
            
            for config in configs {
                let key = config["config_key"].as_str().unwrap_or("N/A");
                let value = &config["config_value"];
                
                result.push_str(&format!("{}: {}\n", key, serde_json::to_string(value).unwrap_or("N/A".to_string())));
            }
            
            result
        }
        Err(e) => format!("❌ 获取系统配置失败: {}", e)
    }
}

async fn get_user_context(state: &AppState, user_id: i64) -> String {
    match state.db.get_user_context(user_id).await {
        Ok(Some(context)) => {
            format!(
                "👤 用户 {} 的上下文:\n\n{}",
                user_id,
                serde_json::to_string_pretty(&context).unwrap_or("无法格式化上下文数据".to_string())
            )
        }
        Ok(None) => format!("📭 用户 {} 没有找到上下文信息。", user_id),
        Err(e) => format!("❌ 获取用户 {} 上下文失败: {}", user_id, e)
    }
}

async fn get_group_context(state: &AppState, group_id: i64) -> String {
    match state.db.get_group_context(group_id).await {
        Ok(Some(context)) => {
            format!(
                "👥 群组 {} 的上下文:\n\n{}",
                group_id,
                serde_json::to_string_pretty(&context).unwrap_or("无法格式化上下文数据".to_string())
            )
        }
        Ok(None) => format!("📭 群组 {} 没有找到上下文信息。", group_id),
        Err(e) => format!("❌ 获取群组 {} 上下文失败: {}", group_id, e)
    }
}

// --- LLM Client ---

#[derive(Serialize)]
struct LLMRequest {
    model: String,
    messages: Vec<LLMMessage>,
}
#[derive(Serialize, Deserialize)]
struct LLMMessage {
    role: String,
    content: String,
}
#[derive(Deserialize)]
struct LLMResponse {
    choices: Vec<LLMChoice>,
}
#[derive(Deserialize)]
struct LLMChoice {
    message: LLMMessage,
}

async fn call_llm(state: &AppState, system: &str, user: &str) -> anyhow::Result<String> {
    let url = format!("{}/chat/completions", state.config.llm_base_url.trim_end_matches('/'));
    
    let body = LLMRequest {
        model: state.config.llm_model.clone(),
        messages: vec![
            LLMMessage { role: "system".to_string(), content: system.to_string() },
            LLMMessage { role: "user".to_string(), content: user.to_string() },
        ],
    };

    let res = state.http_client.post(&url)
        .header("Authorization", format!("Bearer {}", state.config.llm_api_key))
        .json(&body)
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(anyhow::anyhow!("API Error: {}", res.text().await?));
    }

    let data: LLMResponse = res.json().await?;
    data.choices.first().map(|c| c.message.content.clone()).ok_or(anyhow::anyhow!("Empty response"))
}
