use anyhow::{Context, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

/// Application configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Anthropic API key (base64 encoded for basic obfuscation)
    #[serde(default)]
    pub anthropic_api_key: Option<String>,

    /// Default Claude model to use
    #[serde(default = "default_model")]
    pub default_model: String,

    /// Default max tokens for API calls
    #[serde(default = "default_max_tokens")]
    pub default_max_tokens: u32,

    /// Enable API call logging for cost tracking
    #[serde(default = "default_true")]
    pub enable_api_logging: bool,
}

fn default_model() -> String {
    "claude-sonnet-4-20250514".to_string()
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_true() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            anthropic_api_key: None,
            default_model: default_model(),
            default_max_tokens: default_max_tokens(),
            enable_api_logging: true,
        }
    }
}

impl AppConfig {
    /// Get the configuration directory path
    fn get_config_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .context("Failed to get app data directory")?;

        let config_dir = app_data_dir.join("config");
        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;

        Ok(config_dir)
    }

    /// Get the settings file path
    fn get_settings_path(app_handle: &tauri::AppHandle) -> Result<PathBuf> {
        Ok(Self::get_config_dir(app_handle)?.join("settings.json"))
    }

    /// Load configuration from settings file
    pub fn load(app_handle: &tauri::AppHandle) -> Result<Self> {
        let settings_path = Self::get_settings_path(app_handle)?;

        if settings_path.exists() {
            let contents = fs::read_to_string(&settings_path)
                .context("Failed to read settings file")?;

            let config: AppConfig = serde_json::from_str(&contents)
                .context("Failed to parse settings file")?;

            Ok(config)
        } else {
            // Create default config file
            let config = AppConfig::default();
            config.save(app_handle)?;
            Ok(config)
        }
    }

    /// Save configuration to settings file
    pub fn save(&self, app_handle: &tauri::AppHandle) -> Result<()> {
        let settings_path = Self::get_settings_path(app_handle)?;

        let contents = serde_json::to_string_pretty(self)
            .context("Failed to serialize settings")?;

        fs::write(&settings_path, contents)
            .context("Failed to write settings file")?;

        Ok(())
    }

    /// Get the API key from config or environment variable
    /// Priority: 1. Environment variable, 2. Config file
    pub fn get_api_key(&self) -> Result<String> {
        // First try environment variable
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            return Ok(key);
        }

        // Then try config file
        if let Some(encoded_key) = &self.anthropic_api_key {
            // Decode from base64
            let decoded = base64::prelude::BASE64_STANDARD
                .decode(encoded_key)
                .context("Failed to decode API key from config")?;

            let key = String::from_utf8(decoded)
                .context("Invalid UTF-8 in decoded API key")?;

            return Ok(key);
        }

        anyhow::bail!("ANTHROPIC_API_KEY not found in environment or config file")
    }

    /// Set the API key in config (stores as base64)
    pub fn set_api_key(&mut self, api_key: &str) {
        let encoded = base64::prelude::BASE64_STANDARD.encode(api_key.as_bytes());
        self.anthropic_api_key = Some(encoded);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.default_model, "claude-sonnet-4-20250514");
        assert_eq!(config.default_max_tokens, 4096);
        assert!(config.enable_api_logging);
        assert!(config.anthropic_api_key.is_none());
    }

    #[test]
    fn test_api_key_encoding() {
        let mut config = AppConfig::default();
        let test_key = "sk-ant-test-key-12345";

        config.set_api_key(test_key);
        assert!(config.anthropic_api_key.is_some());

        // Verify it's base64 encoded
        let encoded = config.anthropic_api_key.unwrap();
        let decoded = base64::prelude::BASE64_STANDARD.decode(&encoded).unwrap();
        let decoded_str = String::from_utf8(decoded).unwrap();
        assert_eq!(decoded_str, test_key);
    }

    #[test]
    fn test_serialization() {
        let mut config = AppConfig::default();
        config.set_api_key("test-key");

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.default_model, deserialized.default_model);
        assert_eq!(config.anthropic_api_key, deserialized.anthropic_api_key);
    }
}
