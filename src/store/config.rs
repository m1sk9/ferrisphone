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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn sample_toml() -> &'static str {
        r#"
[general]
debug = true
logging = false
memory = true
mods = [123, 456]

[context]
max_chat_context = 500
max_oneline_context = 1000

[model]
claude = ["haiku", "sonnet"]
chatgpt = ["gpt-4.5"]
inception = [""]
gemini = [""]

[system_prompt]
claude = "You are a helpful assistant."
chatgpt = ""
inception = ""
gemini = ""
"#
    }

    #[test]
    fn parse_valid_config() {
        let config: Config = toml::from_str(sample_toml()).unwrap();
        assert!(config.general.debug);
        assert!(!config.general.logging);
        assert!(config.general.memory);
        assert_eq!(config.general.mods, vec![123, 456]);
        assert_eq!(config.context.max_chat_context, 500);
        assert_eq!(config.context.max_oneline_context, 1000);
        assert_eq!(config.model.claude, vec!["haiku", "sonnet"]);
        assert_eq!(config.system_prompt.claude, "You are a helpful assistant.");
    }

    #[test]
    fn load_from_env_path() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(sample_toml().as_bytes()).unwrap();

        // SAFETY: This test runs in isolation; no other threads depend on this env var.
        unsafe {
            std::env::set_var("FERRISPHONE_CONFIG", tmp.path());
        }
        let config = Config::load().unwrap();
        unsafe {
            std::env::remove_var("FERRISPHONE_CONFIG");
        }

        assert!(config.general.debug);
    }

    #[test]
    fn invalid_toml_returns_error() {
        let result: Result<Config, _> = toml::from_str("not valid toml [[[");
        assert!(result.is_err());
    }
}
