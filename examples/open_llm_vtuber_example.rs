use napcat_backend::*;
use std::sync::Arc;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize configuration
    let config_manager = DefaultConfigManager;
    let config = config_manager.default_config();
    
    // Create factories
    let asr_factory = DefaultASRFactory;
    let tts_factory = DefaultTTSFactory;
    let vad_factory = DefaultVADFactory;
    let agent_factory = DefaultAgentFactory;
    let llm_factory = DefaultLLMFactory;
    
    // Create service context
    let mut service_context = ServiceContext::new();
    
    // Initialize components using factories
    let asr_engine = asr_factory.create_asr(&config.character_config.asr_config)?;
    let tts_engine = tts_factory.create_tts(&config.character_config.tts_config)?;
    let vad_engine = vad_factory.create_vad(&config.character_config.vad_config)?;
    let agent_engine = agent_factory.create_agent(&config.character_config.agent_config)?;
    let llm_engine = llm_factory.create_llm(&config)?;
    
    // Add components to service context
    service_context.init_asr(asr_engine);
    service_context.init_tts(tts_engine);
    service_context.init_vad(vad_engine);
    service_context.init_agent(agent_engine);
    service_context.init_llm(llm_engine);
    
    // Initialize MCP client
    let mcp_client = Arc::new(MCPClient::new());
    service_context.init_mcp(mcp_client);
    
    // Example: Using ASR to transcribe audio
    println!("Using ASR engine...");
    let asr = service_context.get_asr()?;
    let mock_audio = vec![0.0; 100]; // Mock audio data
    let transcription = asr.transcribe_np(mock_audio)?;
    println!("Transcription: {}", transcription);
    
    // Example: Using TTS to generate audio
    println!("Using TTS engine...");
    let tts = service_context.get_tts()?;
    let audio_file = tts.generate_audio("Hello, this is a test", Some("test_output"))?;
    println!("Generated audio file: {}", audio_file);
    
    // Example: Using VAD to detect speech
    println!("Using VAD engine...");
    let vad = service_context.get_vad()?;
    let mock_audio_bytes = vec![0u8; 100]; // Mock audio bytes
    let speech_segments = vad.detect_speech(&mock_audio_bytes)?;
    println!("Detected {} speech segments", speech_segments.len());
    
    // Example: Using Agent to process input
    println!("Using Agent engine...");
    let agent = service_context.get_agent()?;
    let input = BaseInput {
        text: "Hello, how are you?".to_string(),
        audio_path: None,
        image_data: None,
    };
    
    let mut response_stream = agent.chat(input).await?;
    while let Some(response) = response_stream.next().await {
        match response {
            Ok(output) => println!("Agent response: {}", output.text),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    
    // Example: Using LLM for chat completion
    println!("Using LLM engine...");
    let llm = service_context.get_llm()?;
    let mut messages = std::collections::HashMap::new();
    messages.insert("role".to_string(), "user".to_string());
    messages.insert("content".to_string(), "Hello, how are you?".to_string());
    
    let mut response_stream = llm.chat_completion(vec![messages], None, None).await?;
    while let Some(response) = response_stream.next().await {
        match response {
            Ok(text) => println!("LLM response: {}", text),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    
    // Example: Using MCP client
    println!("Using MCP client...");
    let mcp = service_context.get_mcp()?;
    
    // Add a mock server
    let server_config = ServerConfig {
        command: "echo".to_string(),
        args: vec!["hello".to_string()],
        env: None,
        cwd: None,
        timeout: Some(10),
    };
    
    mcp.add_server("mock_server".to_string(), server_config).await?;
    println!("Added mock server");
    
    // List tools (would be empty in this mock implementation)
    let tools = mcp.list_tools("mock_server").await?;
    println!("Found {} tools", tools.len());
    
    // Example: Using WebSocket handler
    println!("Creating WebSocket handler...");
    let ws_handler = WebSocketHandler::new();
    
    // Print message types
    println!("Message types:");
    println!("  Text: {:?}", MessageType::Text);
    println!("  Audio: {:?}", MessageType::Audio);
    println!("  DisplayText: {:?}", MessageType::DisplayText);
    
    println!("Open-LLM-VTuber example completed successfully!");
    
    Ok(())
}