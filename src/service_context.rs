use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::asr_interface::ASRInterface;
use crate::tts_interface::TTSInterface;
use crate::vad_interface::VADInterface;
use crate::agent_interface::AgentInterface;
use crate::stateless_llm_interface::StatelessLLMInterface;
use crate::mcp_interface::MCPClient;

/// Service context for managing ASR, TTS, VAD, Agent, and other components
pub struct ServiceContext {
    /// ASR engine instance
    pub asr_engine: Option<Arc<dyn ASRInterface>>,
    /// TTS engine instance
    pub tts_engine: Option<Arc<dyn TTSInterface>>,
    /// VAD engine instance
    pub vad_engine: Option<Arc<dyn VADInterface>>,
    /// Agent engine instance
    pub agent_engine: Option<Arc<dyn AgentInterface>>,
    /// Stateless LLM instance
    pub llm_engine: Option<Arc<dyn StatelessLLMInterface>>,
    /// MCP client instance
    pub mcp_client: Option<Arc<MCPClient>>,
    /// System prompt
    pub system_prompt: Option<String>,
    /// History UID
    pub history_uid: Option<String>,
}

impl ServiceContext {
    /// Create a new service context
    pub fn new() -> Self {
        Self {
            asr_engine: None,
            tts_engine: None,
            vad_engine: None,
            agent_engine: None,
            llm_engine: None,
            mcp_client: None,
            system_prompt: None,
            history_uid: None,
        }
    }

    /// Initialize ASR engine
    pub fn init_asr(&mut self, asr_engine: Arc<dyn ASRInterface>) {
        self.asr_engine = Some(asr_engine);
    }

    /// Initialize TTS engine
    pub fn init_tts(&mut self, tts_engine: Arc<dyn TTSInterface>) {
        self.tts_engine = Some(tts_engine);
    }

    /// Initialize VAD engine
    pub fn init_vad(&mut self, vad_engine: Arc<dyn VADInterface>) {
        self.vad_engine = Some(vad_engine);
    }

    /// Initialize Agent engine
    pub fn init_agent(&mut self, agent_engine: Arc<dyn AgentInterface>) {
        self.agent_engine = Some(agent_engine);
    }

    /// Initialize Stateless LLM engine
    pub fn init_llm(&mut self, llm_engine: Arc<dyn StatelessLLMInterface>) {
        self.llm_engine = Some(llm_engine);
    }

    /// Initialize MCP client
    pub fn init_mcp(&mut self, mcp_client: Arc<MCPClient>) {
        self.mcp_client = Some(mcp_client);
    }

    /// Get ASR engine reference
    pub fn get_asr(&self) -> Result<Arc<dyn ASRInterface>> {
        self.asr_engine
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("ASR engine not initialized"))
    }

    /// Get TTS engine reference
    pub fn get_tts(&self) -> Result<Arc<dyn TTSInterface>> {
        self.tts_engine
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("TTS engine not initialized"))
    }

    /// Get VAD engine reference
    pub fn get_vad(&self) -> Result<Arc<dyn VADInterface>> {
        self.vad_engine
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("VAD engine not initialized"))
    }

    /// Get Agent engine reference
    pub fn get_agent(&self) -> Result<Arc<dyn AgentInterface>> {
        self.agent_engine
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Agent engine not initialized"))
    }

    /// Get Stateless LLM engine reference
    pub fn get_llm(&self) -> Result<Arc<dyn StatelessLLMInterface>> {
        self.llm_engine
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("LLM engine not initialized"))
    }

    /// Get MCP client reference
    pub fn get_mcp(&self) -> Result<Arc<MCPClient>> {
        self.mcp_client
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("MCP client not initialized"))
    }
}