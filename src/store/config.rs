use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub context: ContextConfig,
    pub model: ModelConfig,
    pub system_prompt: SystemPromptConfig,
}

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    pub debug: bool,
    pub logging: bool,
    pub memory: bool,
    pub mods: Vec<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ContextConfig {
    pub max_chat_context: usize,
    pub max_oneline_context: usize,
}

#[derive(Debug, Deserialize)]
pub struct ModelConfig {
    pub claude: Vec<String>,
    pub chatgpt: Vec<String>,
    pub inception: Vec<String>,
    pub gemini: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SystemPromptConfig {
    pub claude: String,
    pub chatgpt: String,
    pub inception: String,
    pub gemini: String,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = std::env::var("FERRISPHONE_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./ferrisphone.toml"));
        let content = std::fs::read_to_string(&path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
}
