use anyhow::Result;
use std::sync::Arc;
use std::pin::Pin;
use std::collections::HashMap;

use crate::asr_interface::ASRInterface;
use crate::tts_interface::TTSInterface;
use crate::vad_interface::VADInterface;
use crate::agent_interface::{AgentInterface, BaseInput, BaseOutput};
use crate::stateless_llm_interface::StatelessLLMInterface;
use crate::config_manager::{Config, ASRConfig, TTSConfig, VADConfig, AgentConfig};

/// ASR Factory trait for creating ASR system instances
pub trait ASRFactory: Send + Sync {
    /// Create an ASR instance based on configuration
    fn create_asr(&self, config: &ASRConfig) -> Result<Arc<dyn ASRInterface>>;
}

/// TTS Factory trait for creating TTS engine instances
pub trait TTSFactory: Send + Sync {
    /// Create a TTS instance based on configuration
    fn create_tts(&self, config: &TTSConfig) -> Result<Arc<dyn TTSInterface>>;
}

/// VAD Factory trait for creating VAD engine instances
pub trait VADFactory: Send + Sync {
    /// Create a VAD instance based on configuration
    fn create_vad(&self, config: &VADConfig) -> Result<Arc<dyn VADInterface>>;
}

/// Agent Factory trait for creating Agent instances
pub trait AgentFactory: Send + Sync {
    /// Create an Agent instance based on configuration
    fn create_agent(&self, config: &AgentConfig) -> Result<Arc<dyn AgentInterface>>;
}

/// LLM Factory trait for creating LLM instances
pub trait LLMFactory: Send + Sync {
    /// Create an LLM instance based on configuration
    fn create_llm(&self, config: &Config) -> Result<Arc<dyn StatelessLLMInterface>>;
}

/// Default implementations for the factory traits
pub struct DefaultASRFactory;
pub struct DefaultTTSFactory;
pub struct DefaultVADFactory;
pub struct DefaultAgentFactory;
pub struct DefaultLLMFactory;

// Mock implementations defined at module level to avoid lifetime issues
#[derive(Clone)]
struct MockASR;

unsafe impl Send for MockASR {}
unsafe impl Sync for MockASR {}

impl ASRInterface for MockASR {
    fn transcribe_np(&self, _audio: Vec<f32>) -> Result<String> {
        Ok("Mock transcription".to_string())
    }
}

#[derive(Clone)]
struct MockTTS;

unsafe impl Send for MockTTS {}
unsafe impl Sync for MockTTS {}

impl TTSInterface for MockTTS {
    fn generate_audio(&self, text: &str, file_name_no_ext: Option<&str>) -> Result<String> {
        // Generate a mock file path
        let file_name = match file_name_no_ext {
            Some(name) => format!("{}.wav", name),
            None => "output.wav".to_string(),
        };
        
        // In a real implementation, this would generate actual audio
        println!("Mock TTS: Would generate audio for text: {}", text);
        
        Ok(file_name)
    }
}

#[derive(Clone)]
struct MockVAD;

unsafe impl Send for MockVAD {}
unsafe impl Sync for MockVAD {}

impl VADInterface for MockVAD {
    fn detect_speech(&self, audio_data: &[u8]) -> Result<Vec<Vec<u8>>> {
        // In a real implementation, this would detect speech in the audio data
        // For now, return the original audio as a single segment
        println!("Mock VAD: Detecting speech in {} bytes of audio", audio_data.len());
        
        // Return the original audio as a single segment if it's not empty
        if !audio_data.is_empty() {
            Ok(vec![audio_data.to_vec()])
        } else {
            Ok(vec![])
        }
    }
}

#[derive(Clone)]
struct MockAgent;

unsafe impl Send for MockAgent {}
unsafe impl Sync for MockAgent {}

#[async_trait::async_trait]
impl AgentInterface for MockAgent {
    async fn chat(&self, input_data: BaseInput) -> Result<Pin<Box<dyn futures::Stream<Item = Result<BaseOutput>> + Send>>> {
        // In a real implementation, this would process the input and generate responses
        println!("Mock Agent: Processing input: {}", input_data.text);
        
        // Create a mock response
        let output = BaseOutput {
            text: format!("Echo: {}", input_data.text),
            audio_path: None,
            actions: None,
        };
        
        // Create a stream with the single output
        let stream = futures::stream::iter(vec![Ok(output)]);
        
        Ok(Box::pin(stream))
    }
    
    fn handle_interrupt(&self, heard_response: &str) -> Result<()> {
        println!("Mock Agent: Handling interrupt: {}", heard_response);
        Ok(())
    }
    
    fn set_memory_from_history(&self, conf_uid: &str, history_uid: &str) -> Result<()> {
        println!("Mock Agent: Setting memory from history - conf_uid: {}, history_uid: {}", conf_uid, history_uid);
        Ok(())
    }
}

#[derive(Clone)]
struct MockLLM;

unsafe impl Send for MockLLM {}
unsafe impl Sync for MockLLM {}

#[async_trait::async_trait]
impl StatelessLLMInterface for MockLLM {
    async fn chat_completion(
        &self,
        messages: Vec<HashMap<String, String>>,
        system: Option<String>,
        tools: Option<Vec<HashMap<String, serde_json::Value>>>,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<String>> + Send>>> {
        println!("Mock LLM: Processing {} messages", messages.len());
        
        // Create a mock response
        let response = "Mock LLM response".to_string();
        
        // Create a stream with the single response
        let stream = futures::stream::iter(vec![Ok(response)]);
        
        Ok(Box::pin(stream))
    }
}

impl ASRFactory for DefaultASRFactory {
    fn create_asr(&self, config: &ASRConfig) -> Result<Arc<dyn ASRInterface>> {
        // In a real implementation, this would create an actual ASR instance based on the config
        // For now, we'll return a mock implementation
        Ok(Arc::new(MockASR))
    }
}

impl TTSFactory for DefaultTTSFactory {
    fn create_tts(&self, config: &TTSConfig) -> Result<Arc<dyn TTSInterface>> {
        // In a real implementation, this would create an actual TTS instance based on the config
        // For now, we'll return a mock implementation
        Ok(Arc::new(MockTTS))
    }
}

impl VADFactory for DefaultVADFactory {
    fn create_vad(&self, config: &VADConfig) -> Result<Arc<dyn VADInterface>> {
        // In a real implementation, this would create an actual VAD instance based on the config
        // For now, we'll return a mock implementation
        Ok(Arc::new(MockVAD))
    }
}

impl AgentFactory for DefaultAgentFactory {
    fn create_agent(&self, config: &AgentConfig) -> Result<Arc<dyn AgentInterface>> {
        // In a real implementation, this would create an actual Agent instance based on the config
        // For now, we'll return a mock implementation
        Ok(Arc::new(MockAgent))
    }
}

impl LLMFactory for DefaultLLMFactory {
    fn create_llm(&self, config: &Config) -> Result<Arc<dyn StatelessLLMInterface>> {
        // In a real implementation, this would create an actual LLM instance based on the config
        // For now, we'll return a mock implementation
        Ok(Arc::new(MockLLM))
    }
}
