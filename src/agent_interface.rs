use anyhow::Result;
use futures::Stream;
use std::pin::Pin;

/// Base input type for agent
#[derive(Debug, Clone)]
pub struct BaseInput {
    pub text: String,
    pub audio_path: Option<String>,
    pub image_data: Option<Vec<u8>>,
}

/// Base output type for agent
#[derive(Debug, Clone)]
pub struct BaseOutput {
    pub text: String,
    pub audio_path: Option<String>,
    pub actions: Option<Vec<String>>,
}

/// Agent Interface
/// Provides functionality for conversational agents
#[async_trait::async_trait]
pub trait AgentInterface: Send + Sync {
    /// Asynchronously chat with the agent
    async fn chat(&self, input_data: BaseInput) -> Result<Pin<Box<dyn Stream<Item = Result<BaseOutput>> + Send>>>;

    /// Handle user interruption
    fn handle_interrupt(&self, heard_response: &str) -> Result<()>;

    /// Set memory from chat history
    fn set_memory_from_history(&self, conf_uid: &str, history_uid: &str) -> Result<()>;
}