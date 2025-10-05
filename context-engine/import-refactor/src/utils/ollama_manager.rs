// Ollama server management for automatic startup and process control

use anyhow::{Context, Result};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;
use which::which;

/// Ollama server manager for automatic startup and control
pub struct OllamaManager {
    host: String,
    port: u16,
    process: Option<Child>,
    auto_started: bool,
}

impl OllamaManager {
    /// Create a new Ollama manager with default settings
    pub fn new() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 11434,
            process: None,
            auto_started: false,
        }
    }

    /// Create with custom host and port
    pub fn with_host(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            process: None,
            auto_started: false,
        }
    }

    /// Parse host:port from a string (e.g., "localhost:11434")
    pub fn from_host_string(host_string: &str) -> Result<Self> {
        if let Some((host, port_str)) = host_string.split_once(':') {
            let port = port_str.parse::<u16>()
                .context("Invalid port number in host string")?;
            Ok(Self::with_host(host.to_string(), port))
        } else {
            // Assume it's just a host, use default port
            Ok(Self::with_host(host_string.to_string(), 11434))
        }
    }

    /// Get the base URL for API calls
    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }

    /// Check if Ollama server is running and accessible
    pub async fn is_running(&self) -> bool {
        let client = reqwest::Client::new();
        let url = format!("{}/api/tags", self.base_url());
        
        match client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Check if we can start Ollama (binary exists)
    pub fn can_start_ollama() -> bool {
        which("ollama").is_ok()
    }

    /// Start Ollama server if not already running
    pub async fn ensure_running(&mut self) -> Result<()> {
        // If already running, we're good
        if self.is_running().await {
            println!("✅ Ollama server already running at {}", self.base_url());
            return Ok(());
        }

        // Only try to start if it's localhost (don't start remote servers)
        if self.host != "localhost" && self.host != "127.0.0.1" {
            return Err(anyhow::anyhow!(
                "Ollama server not accessible at {} and cannot auto-start remote servers. \
                Please start Ollama manually on the remote host.",
                self.base_url()
            ));
        }

        // Check if Ollama binary is available
        if !Self::can_start_ollama() {
            return Err(anyhow::anyhow!(
                "Ollama binary not found in PATH. Please install Ollama from https://ollama.ai \
                or specify a different host with --ollama-host"
            ));
        }

        println!("🚀 Starting Ollama server...");
        self.start_server().await?;
        
        // Wait for server to be ready
        self.wait_for_ready().await?;
        
        println!("✅ Ollama server started successfully at {}", self.base_url());
        Ok(())
    }

    /// Start the Ollama server process
    async fn start_server(&mut self) -> Result<()> {
        let mut cmd = Command::new("ollama");
        cmd.arg("serve")
            .stdout(Stdio::null())  // Suppress output
            .stderr(Stdio::null())
            .stdin(Stdio::null());

        // Set custom port if not default
        if self.port != 11434 {
            cmd.env("OLLAMA_PORT", self.port.to_string());
        }

        let child = cmd.spawn()
            .context("Failed to start Ollama server. Make sure 'ollama' is installed and in PATH.")?;

        self.process = Some(child);
        self.auto_started = true;
        
        Ok(())
    }

    /// Wait for the server to be ready to accept connections
    async fn wait_for_ready(&self) -> Result<()> {
        let max_attempts = 30;  // 30 seconds timeout
        let mut attempts = 0;

        while attempts < max_attempts {
            if self.is_running().await {
                return Ok(());
            }
            
            sleep(Duration::from_secs(1)).await;
            attempts += 1;
        }

        Err(anyhow::anyhow!(
            "Ollama server failed to start within 30 seconds. Check that Ollama is properly installed."
        ))
    }

    /// Stop the Ollama server if we started it
    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            if self.auto_started {
                println!("🛑 Stopping auto-started Ollama server...");
                
                // Try graceful shutdown first
                if let Err(_) = process.kill() {
                    // Force kill if graceful shutdown fails
                    let _ = process.wait();
                }
                
                self.auto_started = false;
                println!("✅ Ollama server stopped");
            }
        }
        Ok(())
    }

    /// Get server status information
    pub async fn get_status(&self) -> ServerStatus {
        let running = self.is_running().await;
        let can_start = Self::can_start_ollama();
        
        ServerStatus {
            url: self.base_url(),
            running,
            auto_started: self.auto_started,
            can_start_local: can_start && (self.host == "localhost" || self.host == "127.0.0.1"),
        }
    }

    /// List available models on the server
    pub async fn list_models(&self) -> Result<Vec<String>> {
        if !self.is_running().await {
            return Err(anyhow::anyhow!("Ollama server not running at {}", self.base_url()));
        }

        let client = reqwest::Client::new();
        let url = format!("{}/api/tags", self.base_url());
        
        let response = client
            .get(&url)
            .send()
            .await
            .context("Failed to get model list from Ollama")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Ollama API returned error: {}", response.status()));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        let models = json["models"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|model| model["name"].as_str())
            .map(|name| name.to_string())
            .collect();

        Ok(models)
    }

    /// Check if a specific model is available
    pub async fn has_model(&self, model_name: &str) -> Result<bool> {
        let models = self.list_models().await?;
        Ok(models.iter().any(|m| m.starts_with(model_name)))
    }

    /// Recommend a model to pull if none are available
    pub fn recommend_model() -> &'static str {
        "codellama:7b"
    }

    /// Provide instructions for manual setup
    pub fn setup_instructions(&self) -> String {
        if Self::can_start_ollama() {
            format!(
                r#"Ollama is installed but server not running. To start manually:

1. Start Ollama:
   ollama serve

2. Pull a code model:
   ollama pull {}

3. Then retry your analysis command"#,
                Self::recommend_model()
            )
        } else {
            format!(
                r#"Ollama not found. To install:

1. Install Ollama:
   - Windows/Mac: Download from https://ollama.ai
   - Linux: curl -fsSL https://ollama.ai/install.sh | sh

2. Start Ollama:
   ollama serve{}

3. Pull a code model:
   ollama pull {}

4. Then retry your analysis command"#,
                if self.port != 11434 { &format!(" --port {}", self.port) } else { "" },
                Self::recommend_model()
            )
        }
    }
}

impl Drop for OllamaManager {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[derive(Debug)]
pub struct ServerStatus {
    pub url: String,
    pub running: bool,
    pub auto_started: bool,
    pub can_start_local: bool,
}

impl ServerStatus {
    pub fn print_status(&self) {
        println!("🔍 Ollama Server Status:");
        println!("  URL: {}", self.url);
        println!("  Running: {}", if self.running { "✅ Yes" } else { "❌ No" });
        
        if self.auto_started {
            println!("  Auto-started: ✅ Yes (will be stopped automatically)");
        }
        
        if !self.running && self.can_start_local {
            println!("  Can auto-start: ✅ Yes");
        } else if !self.running {
            println!("  Can auto-start: ❌ No (remote host or Ollama not installed)");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_parsing() {
        let manager = OllamaManager::from_host_string("localhost:8080").unwrap();
        assert_eq!(manager.host, "localhost");
        assert_eq!(manager.port, 8080);
        assert_eq!(manager.base_url(), "http://localhost:8080");

        let manager = OllamaManager::from_host_string("remote-server").unwrap();
        assert_eq!(manager.host, "remote-server");
        assert_eq!(manager.port, 11434);
        assert_eq!(manager.base_url(), "http://remote-server:11434");
    }

    #[test]
    fn test_can_start_detection() {
        let can_start = OllamaManager::can_start_ollama();
        println!("Can start Ollama: {}", can_start);
        // This test will pass regardless, just prints the result
    }

    #[tokio::test]
    async fn test_server_status() {
        let manager = OllamaManager::new();
        let status = manager.get_status().await;
        status.print_status();
    }

    #[test]
    fn test_setup_instructions() {
        let manager = OllamaManager::new();
        let instructions = manager.setup_instructions();
        println!("Setup instructions:\n{}", instructions);
        assert!(instructions.contains("ollama"));
    }
}