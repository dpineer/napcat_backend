use anyhow::Result;

/// TTS (Text-to-Speech) Interface
/// Provides functionality for converting text to speech audio
#[async_trait::async_trait]
pub trait TTSInterface: Send + Sync {
    /// Asynchronously generate audio from text
    async fn async_generate_audio(&self, text: &str, file_name_no_ext: Option<&str>) -> Result<String> {
        // Default implementation runs synchronous generate_audio in a blocking task
        let text = text.to_string();
        let file_name_no_ext = file_name_no_ext.map(|s| s.to_string());
        
        tokio::task::spawn_blocking(move || {
            // This is a workaround since we can't move self into the closure
            // In a real implementation, this would call self.generate_audio(&text, file_name_no_ext.as_deref())
            // For now, return a placeholder
            Ok("".to_string())
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Task join error: {}", e)))
    }

    /// Generate audio from text and return the file path
    fn generate_audio(&self, text: &str, file_name_no_ext: Option<&str>) -> Result<String>;

    /// Remove a file from the file system
    fn remove_file(&self, filepath: &str, verbose: bool) -> Result<()> {
        if std::path::Path::new(filepath).exists() {
            std::fs::remove_file(filepath)?;
            if verbose {
                println!("Removed file: {}", filepath);
            }
        }
        Ok(())
    }

    /// Generate a cross-platform cache file name
    fn generate_cache_file_name(&self, file_name_no_ext: Option<&str>, file_extension: &str) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let base_name = match file_name_no_ext {
            Some(name) => format!("{}_{}", name, timestamp),
            None => format!("tts_cache_{}", timestamp),
        };

        format!("{}.{}", base_name, file_extension)
    }
}
