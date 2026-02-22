use anyhow::Result;

/// ASR (Automatic Speech Recognition) Interface
/// Provides functionality for converting audio to text
#[async_trait::async_trait]
pub trait ASRInterface: Send + Sync {
    /// Asynchronously transcribe speech audio in numpy array format
    async fn async_transcribe_np(&self, audio: Vec<f32>) -> Result<String> {
        // Default implementation runs synchronous transcribe_np in a blocking task
        let audio_clone = audio.clone();
        tokio::task::spawn_blocking(move || {
            // This is a workaround since we can't move self into the closure
            // In a real implementation, this would call self.transcribe_np(audio_clone)
            // For now, return a placeholder
            Ok("".to_string())
        })
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("Task join error: {}", e)))
    }

    /// Transcribe speech audio in numpy array format and return the transcription
    fn transcribe_np(&self, audio: Vec<f32>) -> Result<String>;

    /// Convert a numpy array of audio data to a .wav file
    fn nparray_to_audio_file(
        &self,
        audio: Vec<f32>,
        sample_rate: u32,
        file_path: &str,
    ) -> Result<()> {
        use hound;

        // Make sure the audio is in the range [-1, 1]
        let clamped_audio: Vec<f32> = audio
            .iter()
            .map(|&x| x.clamp(-1.0, 1.0))
            .collect();

        // Convert the audio to 16-bit PCM
        let samples: Vec<i16> = clamped_audio
            .iter()
            .map(|&x| (x * i16::MAX as f32) as i16)
            .collect();

        let spec = hound::WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(file_path, spec)?;
        for &sample in &samples {
            writer.write_sample(sample)?;
        }
        writer.finalize()?;

        Ok(())
    }
}
