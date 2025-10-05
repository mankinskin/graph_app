// Configuration management for Candle LLM server
// Handles server settings, model configurations, and system capabilities

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Server configuration for the Candle LLM server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub model: ModelConfig,
    pub server: ServerSettings,
    pub cache: CacheConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_id: String,
    pub device: String, // "cpu", "cuda", "metal", "auto"
    pub temperature: f64,
    pub top_p: f64,
    pub max_tokens: usize,
    pub context_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    pub max_concurrent_requests: usize,
    pub request_timeout_seconds: u64,
    pub enable_cors: bool,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub cache_dir: PathBuf,
    pub max_cache_size_gb: f64,
    pub cleanup_old_models: bool,
    pub model_ttl_days: u32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            model: ModelConfig::default(),
            server: ServerSettings::default(),
            cache: CacheConfig::default(),
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_id: "microsoft/CodeLlama-7b-Instruct-hf".to_string(),
            device: "auto".to_string(),
            temperature: 0.1,
            top_p: 0.9,
            max_tokens: 2048,
            context_length: 4096,
        }
    }
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 4,
            request_timeout_seconds: 300, // 5 minutes
            enable_cors: true,
            log_level: "info".to_string(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        let cache_dir = dirs::cache_dir()
            .map(|d| d.join("candle_models"))
            .unwrap_or_else(|| PathBuf::from("./models"));

        Self {
            cache_dir,
            max_cache_size_gb: 50.0,
            cleanup_old_models: true,
            model_ttl_days: 30,
        }
    }
}

impl ServerConfig {
    /// Load configuration from file or create default
    pub fn load_or_default(config_path: Option<&PathBuf>) -> Result<Self> {
        if let Some(path) = config_path {
            if path.exists() {
                let content = std::fs::read_to_string(path)
                    .with_context(|| format!("Failed to read config file: {:?}", path))?;
                
                let config: ServerConfig = toml::from_str(&content)
                    .with_context(|| format!("Failed to parse config file: {:?}", path))?;
                
                Ok(config)
            } else {
                let default_config = Self::default();
                default_config.save(path)?;
                Ok(default_config)
            }
        } else {
            Ok(Self::default())
        }
    }

    /// Save configuration to file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;
        
        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate port range
        if self.port == 0 {
            return Err(anyhow::anyhow!("Port cannot be 0"));
        }

        // Validate model settings
        if self.model.temperature < 0.0 || self.model.temperature > 2.0 {
            return Err(anyhow::anyhow!("Temperature must be between 0.0 and 2.0"));
        }

        if self.model.top_p < 0.0 || self.model.top_p > 1.0 {
            return Err(anyhow::anyhow!("top_p must be between 0.0 and 1.0"));
        }

        if self.model.max_tokens == 0 {
            return Err(anyhow::anyhow!("max_tokens must be greater than 0"));
        }

        // Validate device
        match self.model.device.to_lowercase().as_str() {
            "cpu" | "cuda" | "metal" | "auto" => {},
            _ => return Err(anyhow::anyhow!("Device must be one of: cpu, cuda, metal, auto")),
        }

        // Validate cache settings
        if self.cache.max_cache_size_gb <= 0.0 {
            return Err(anyhow::anyhow!("max_cache_size_gb must be positive"));
        }

        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_config_validation() {
        let mut config = ServerConfig::default();
        
        // Test invalid temperature
        config.model.temperature = -1.0;
        assert!(config.validate().is_err());
        
        config.model.temperature = 0.1; // Fix it
        assert!(config.validate().is_ok());
        
        // Test invalid port
        config.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_save_load() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_path = temp_file.path().to_path_buf();
        
        let original_config = ServerConfig::default();
        original_config.save(&config_path).unwrap();
        
        let loaded_config = ServerConfig::load_or_default(Some(&config_path)).unwrap();
        assert_eq!(original_config.host, loaded_config.host);
        assert_eq!(original_config.port, loaded_config.port);
    }
}