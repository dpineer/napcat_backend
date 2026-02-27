use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::asr_interface::ASRInterface;
use crate::tts_interface::TTSInterface;
use crate::vad_interface::VADInterface;
use crate::agent_interface::AgentInterface;
use crate::stateless_llm_interface::StatelessLLMInterface;
use crate::mcp_interface::MCPClient;

#[derive(Clone)]
pub struct ServiceContext {
    /// ASR engine instance
    asr: Option<Arc<dyn ASRInterface>>,
    /// TTS engine instance
    tts: Option<Arc<dyn TTSInterface>>,
    /// VAD engine instance
    vad: Option<Arc<dyn VADInterface>>,
    /// Agent instance
    agent: Option<Arc<dyn AgentInterface>>,
    /// LLM instance
    llm: Option<Arc<dyn StatelessLLMInterface>>,
    /// MCP client instance
    mcp: Option<Arc<MCPClient>>,
}

impl ServiceContext {
    /// Create a new service context
    pub fn new() -> Self {
        Self {
            asr: None,
            tts: None,
            vad: None,
            agent: None,
            llm: None,
            mcp: None,
        }
    }

    /// Initialize ASR engine
    pub fn init_asr(&mut self, asr: Arc<dyn ASRInterface>) {
        self.asr = Some(asr);
    }

    /// Initialize TTS engine
    pub fn init_tts(&mut self, tts: Arc<dyn TTSInterface>) {
        self.tts = Some(tts);
    }

    /// Initialize VAD engine
    pub fn init_vad(&mut self, vad: Arc<dyn VADInterface>) {
        self.vad = Some(vad);
    }

    /// Initialize Agent engine
    pub fn init_agent(&mut self, agent: Arc<dyn AgentInterface>) {
        self.agent = Some(agent);
    }

    /// Initialize Stateless LLM engine
    pub fn init_llm(&mut self, llm: Arc<dyn StatelessLLMInterface>) {
        self.llm = Some(llm);
    }

    /// Initialize MCP client
    pub fn init_mcp(&mut self, mcp: Arc<MCPClient>) {
        self.mcp = Some(mcp);
    }

    /// Get ASR engine reference
    pub fn get_asr(&self) -> Result<Arc<dyn ASRInterface>> {
        self.asr
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("ASR engine not initialized"))
    }

    /// Get TTS engine reference
    pub fn get_tts(&self) -> Result<Arc<dyn TTSInterface>> {
        self.tts
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("TTS engine not initialized"))
    }

    /// Get VAD engine reference
    pub fn get_vad(&self) -> Result<Arc<dyn VADInterface>> {
        self.vad
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("VAD engine not initialized"))
    }

    /// Get Agent engine reference
    pub fn get_agent(&self) -> Result<Arc<dyn AgentInterface>> {
        self.agent
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Agent engine not initialized"))
    }

    /// Get Stateless LLM engine reference
    pub fn get_llm(&self) -> Result<Arc<dyn StatelessLLMInterface>> {
        self.llm
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("LLM engine not initialized"))
    }

    /// Get MCP client reference
    pub fn get_mcp(&self) -> Result<Arc<MCPClient>> {
        self.mcp
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("MCP client not initialized"))
    }
}