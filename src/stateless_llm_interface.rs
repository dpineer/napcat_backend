use anyhow::Result;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::collections::HashMap;

/// Stateless LLM (Large Language Model) Interface
/// Provides functionality for chat completion without maintaining conversation state
#[async_trait::async_trait]
pub trait StatelessLLMInterface: Send + Sync {
    /// Asynchronously generate chat completion and return an iterator of responses
    async fn chat_completion(
        &self,
        messages: Vec<HashMap<String, String>>,
        system: Option<String>,
        tools: Option<Vec<HashMap<String, serde_json::Value>>>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>>;
}