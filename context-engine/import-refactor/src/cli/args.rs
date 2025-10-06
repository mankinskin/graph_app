#[cfg(not(test))]
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Name of the source crate (A) that will export items
    #[arg(short = 'a', long = "source-crate", alias = "source")]
    pub source_crate: Option<String>,

    /// Name of the target crate (B) that imports from source crate
    #[arg(short = 'b', long = "target-crate", alias = "target")]
    pub target_crate: Option<String>,

    /// Self-refactor mode: refactor internal crate:: imports within a single crate
    #[arg(
        long = "self",
        help = "Refactor crate:: imports within the specified crate to root-level exports"
    )]
    pub self_refactor: bool,

    /// Run duplication analyzer on the codebase
    #[arg(
        long = "analyze",
        help = "Analyze the codebase for duplicate and similar functions"
    )]
    pub analyze: bool,

    /// Start the Candle LLM server
    #[arg(
        long = "serve",
        help = "Start the local Candle LLM server for hosting models"
    )]
    pub serve: bool,

    /// Download model for the Candle server
    #[arg(
        long = "download-model",
        help = "Download a model for the Candle server"
    )]
    pub download_model: Option<String>,

    /// List available models and system compatibility
    #[arg(
        long = "list-models",
        help = "List recommended models and check system compatibility"
    )]
    pub list_models: bool,

    /// Generate default configuration file
    #[arg(
        long = "init-config",
        help = "Generate a default candle-server.toml configuration file"
    )]
    pub init_config: bool,

    /// Configuration file path
    #[arg(
        long = "config",
        help = "Path to configuration file (default: candle-server.toml)"
    )]
    pub config_file: Option<PathBuf>,

    /// Server host address
    #[arg(
        long = "host",
        help = "Host address for the Candle server",
        default_value = "127.0.0.1"
    )]
    pub host: String,

    /// Server port
    #[arg(
        long = "port",
        help = "Port for the Candle server",
        default_value = "8080"
    )]
    pub port: u16,

    /// Enable AI-powered semantic analysis (requires API key)
    #[arg(
        long = "ai",
        help = "Enable AI-powered semantic code analysis for better duplication detection"
    )]
    pub enable_ai: bool,

    /// AI provider to use (openai, claude, ollama, embedded, auto)
    #[arg(
        long = "ai-provider",
        help = "AI provider: openai, claude, ollama, embedded, or auto (detect from environment)",
        default_value = "embedded"
    )]
    pub ai_provider: String,

    /// AI model to use (e.g., gpt-4, claude-3-5-sonnet-20241022)
    #[arg(
        long = "ai-model",
        help = "Specific AI model to use (optional, uses provider default)"
    )]
    pub ai_model: Option<String>,

    /// Ollama server host and port (e.g., localhost:11434)
    #[arg(
        long = "ollama-host",
        help = "Ollama server host:port (auto-starts local Ollama if not specified)",
        default_value = "localhost:11434"
    )]
    pub ollama_host: String,

    /// Maximum number of functions to analyze with AI (to control costs)
    #[arg(
        long = "ai-max-functions",
        help = "Maximum number of functions to analyze with AI",
        default_value = "20"
    )]
    pub ai_max_functions: usize,

    /// Positional arguments: [SOURCE_CRATE] [TARGET_CRATE] or [CRATE] when using --self
    #[arg(
        help = "Positional arguments: [SOURCE_CRATE] [TARGET_CRATE] or [CRATE] when using --self"
    )]
    pub positional: Vec<String>,

    /// Workspace root directory
    #[arg(short = 'w', long, default_value = ".")]
    pub workspace_root: PathBuf,

    /// Dry run - show what would be changed without modifying files
    #[arg(long)]
    pub dry_run: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

impl Args {
    /// Get the source crate name, preferring the flag over the positional argument
    #[cfg(not(test))]
    pub fn get_source_crate(&self) -> Result<String> {
        if let Some(source) = &self.source_crate {
            Ok(source.clone())
        } else if !self.positional.is_empty() {
            Ok(self.positional[0].clone())
        } else {
            Err(anyhow::anyhow!("Source crate must be specified either via --source-crate/--source flag or as the first positional argument"))
        }
    }

    /// Get the target crate name, preferring the flag over the positional argument
    #[cfg(not(test))]
    pub fn get_target_crate(&self) -> Result<String> {
        if let Some(target) = &self.target_crate {
            Ok(target.clone())
        } else if self.positional.len() >= 2 {
            Ok(self.positional[1].clone())
        } else {
            Err(anyhow::anyhow!("Target crate must be specified either via --target-crate/--target flag or as the second positional argument"))
        }
    }

    /// Get the crate name for self-refactor mode
    #[cfg(not(test))]
    pub fn get_self_crate(&self) -> Result<String> {
        if let Some(source) = &self.source_crate {
            Ok(source.clone())
        } else if !self.positional.is_empty() {
            Ok(self.positional[0].clone())
        } else {
            Err(anyhow::anyhow!("Crate must be specified either via --source-crate/--source flag or as the first positional argument when using --self"))
        }
    }
}