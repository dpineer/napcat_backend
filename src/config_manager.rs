use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// 添加缺失的类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsConfig {
    pub default_type: PromptType,
    pub custom_templates: HashMap<String, CustomTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTemplate {
    pub enabled: bool,
    pub system_prompt: String,
    pub user_prompt: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromptType {
    Chat,
    Professional,
    Creative,
    Analyze,
    Learn,
    Friendly,
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub system_config: SystemConfig,
    pub character_config: CharacterConfig,
    pub prompts: PromptsConfig,  // 添加prompts配置
}

/// System configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub config_alts_dir: String,
    pub tool_prompts: HashMap<String, String>,
    pub cache_dir: String,
    pub audio_cache_dir: String,
}

/// Character configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterConfig {
    pub conf_name: String,
    pub conf_uid: String,
    pub live2d_model_name: String,
    pub character_name: String,
    pub human_name: String,
    pub avatar: String,
    pub persona_prompt: String,
    pub agent_config: AgentConfig,
    pub asr_config: ASRConfig,
    pub tts_config: TTSConfig,
    pub vad_config: VADConfig,
    pub tts_preprocessor_config: TTSPreprocessorConfig,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub conversation_agent_choice: String,
    pub agent_settings: AgentSettings,
    pub llm_configs: LLMConfigs,
}

/// Agent settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSettings {
    pub basic_memory_agent: BasicMemoryAgentSettings,
}

/// Basic memory agent settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicMemoryAgentSettings {
    pub use_mcpp: bool,
    pub mcp_enabled_servers: Vec<String>,
    pub max_history_messages: usize,
    pub enable_memory: bool,
}

/// LLM configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfigs {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    pub base_url: Option<String>,
}

/// ASR configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASRConfig {
    pub asr_model: String,
    pub whisper: WhisperConfig,
}

/// Whisper configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperConfig {
    pub model_size: String,
    pub language: Option<String>,
    pub device: Option<String>,
}

/// TTS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTSConfig {
    pub tts_model: String,
    pub elevenlabs: ElevenLabsConfig,
}

/// ElevenLabs configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevenLabsConfig {
    pub api_key: String,
    pub voice_id: String,
    pub model: Option<String>,
}

/// VAD configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VADConfig {
    pub vad_model: Option<String>,
    pub silero: SileroVADConfig,
}

/// Silero VAD configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SileroVADConfig {
    pub threshold: f32,
    pub min_speech_duration: u64,
    pub min_silence_duration: u64,
}

/// TTS Preprocessor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTSPreprocessorConfig {
    pub translator_config: TranslatorConfig,
    pub expression_extractor_config: ExpressionExtractorConfig,
}

/// Translator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatorConfig {
    pub translate_audio: bool,
    pub translate_provider: String,
    pub source_lang: String,
    pub target_lang: String,
}

/// Expression extractor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionExtractorConfig {
    pub enabled: bool,
    pub model_path: String,
}

/// Configuration manager trait
pub trait ConfigManager: Send + Sync {
    /// Load configuration from file
    fn load_config(&self, config_path: &str) -> Result<Config>;

    /// Save configuration to file
    fn save_config(&self, config: &Config, config_path: &str) -> Result<()>;

    /// Validate configuration
    fn validate_config(&self, config: &Config) -> Result<()>;

    /// Get default configuration
    fn default_config(&self) -> Config;
}

/// Default configuration manager implementation
pub struct DefaultConfigManager;

impl ConfigManager for DefaultConfigManager {
    fn load_config(&self, config_path: &str) -> Result<Config> {
        let content = fs::read_to_string(config_path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    fn save_config(&self, config: &Config, config_path: &str) -> Result<()> {
        let content = serde_yaml::to_string(config)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    fn validate_config(&self, config: &Config) -> Result<()> {
        // Basic validation checks
        if config.character_config.conf_name.is_empty() {
            return Err(anyhow::anyhow!("Configuration name cannot be empty"));
        }
        
        if config.character_config.persona_prompt.is_empty() {
            return Err(anyhow::anyhow!("Persona prompt cannot be empty"));
        }
        
        Ok(())
    }

    fn default_config(&self) -> Config {
        Config {
            system_config: SystemConfig {
                config_alts_dir: "config_alts".to_string(),
                tool_prompts: HashMap::new(),
                cache_dir: "cache".to_string(),
                audio_cache_dir: "cache/audio".to_string(),
            },
            character_config: CharacterConfig {
                conf_name: "default".to_string(),
                conf_uid: uuid::Uuid::new_v4().to_string(),
                live2d_model_name: "default.model3.json".to_string(),
                character_name: "AI Assistant".to_string(),
                human_name: "User".to_string(),
                avatar: "".to_string(),
                persona_prompt: "You are a helpful AI assistant.".to_string(),
                agent_config: AgentConfig {
                    conversation_agent_choice: "basic_memory_agent".to_string(),
                    agent_settings: AgentSettings {
                        basic_memory_agent: BasicMemoryAgentSettings {
                            use_mcpp: false,
                            mcp_enabled_servers: vec![],
                            max_history_messages: 10,
                            enable_memory: true,
                        },
                    },
                    llm_configs: LLMConfigs {
                        provider: "openai".to_string(),
                        model: "gpt-3.5-turbo".to_string(),
                        api_key: "your-api-key".to_string(),
                        base_url: None,
                    },
                },
                asr_config: ASRConfig {
                    asr_model: "whisper".to_string(),
                    whisper: WhisperConfig {
                        model_size: "base".to_string(),
                        language: Some("en".to_string()),
                        device: Some("cpu".to_string()),
                    },
                },
                tts_config: TTSConfig {
                    tts_model: "elevenlabs".to_string(),
                    elevenlabs: ElevenLabsConfig {
                        api_key: "your-api-key".to_string(),
                        voice_id: "default-voice".to_string(),
                        model: Some("eleven_monolingual_v1".to_string()),
                    },
                },
                vad_config: VADConfig {
                    vad_model: Some("silero".to_string()),
                    silero: SileroVADConfig {
                        threshold: 0.5,
                        min_speech_duration: 250,
                        min_silence_duration: 500,
                    },
                },
                tts_preprocessor_config: TTSPreprocessorConfig {
                    translator_config: TranslatorConfig {
                        translate_audio: false,
                        translate_provider: "google".to_string(),
                        source_lang: "en".to_string(),
                        target_lang: "en".to_string(),
                    },
                    expression_extractor_config: ExpressionExtractorConfig {
                        enabled: false,
                        model_path: "models/expression_extractor.onnx".to_string(),
                    },
                },
            },
            prompts: PromptsConfig {
                default_type: PromptType::Chat,
                custom_templates: HashMap::new(),
            },
        }
    }
}

impl Config {
    /// 将自定义模板转换为提示词模板
    pub fn to_prompt_template(&self, custom_template: &CustomTemplate) -> crate::enhanced_prompts::PromptTemplate {
        crate::enhanced_prompts::PromptTemplate {
            description: custom_template.description.clone(),
            system_prompt: custom_template.system_prompt.clone(),
            user_prompt: custom_template.user_prompt.clone(),
        }
    }
}
