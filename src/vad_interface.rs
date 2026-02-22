use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

/// VAD (Voice Activity Detection) Interface
/// Provides functionality for detecting speech activity in audio
pub trait VADInterface: Send + Sync {
    /// Detect speech in audio data and return detected segments
    fn detect_speech(&self, audio_data: &[u8]) -> Result<Vec<Vec<u8>>>;
}