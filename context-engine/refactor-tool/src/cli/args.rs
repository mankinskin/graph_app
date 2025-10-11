#[cfg(not(test))]
use anyhow::Result;
use clap::{
    Parser,
    Subcommand,
};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    /// Workspace root directory
    #[arg(short = 'w', long, default_value = ".", global = true)]
    pub workspace_root: PathBuf,

    /// Dry run - show what would be changed without modifying files
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Refactor import statements between crates
    Imports {
        /// Name of the source crate (A) that will export items
        #[arg(short = 'a', long = "source-crate", alias = "source")]
        source_crate: Option<String>,

        /// Name of the target crate (B) that imports from source crate
        #[arg(short = 'b', long = "target-crate", alias = "target")]
        target_crate: Option<String>,

        /// Self-refactor mode: refactor internal crate:: imports within a single crate
        #[arg(
            long = "self",
            help = "Refactor crate:: imports within the specified crate to root-level exports"
        )]
        self_refactor: bool,

        /// Positional arguments: [SOURCE_CRATE] [TARGET_CRATE] or [CRATE] when using --self
        #[arg(
            help = "Positional arguments: [SOURCE_CRATE] [TARGET_CRATE] or [CRATE] when using --self"
        )]
        positional: Vec<String>,
    },

    /// Analyze the codebase for duplicate and similar functions
    Analyze {
        /// Enable AI-powered semantic analysis (requires API key)
        #[arg(
            long = "ai",
            help = "Enable AI-powered semantic code analysis for better duplication detection"
        )]
        enable_ai: bool,

        /// AI provider to use (openai, claude, ollama, embedded, auto)
        #[arg(
            long = "ai-provider",
            help = "AI provider: openai, claude, ollama, embedded, or auto (detect from environment)",
            default_value = "embedded"
        )]
        ai_provider: String,

        /// AI model to use (e.g., gpt-4, claude-3-5-sonnet-20241022)
        #[arg(
            long = "ai-model",
            help = "Specific AI model to use (optional, uses provider default)"
        )]
        ai_model: Option<String>,

        /// Ollama server host and port (e.g., localhost:11434)
        #[arg(
            long = "ollama-host",
            help = "Ollama server host:port (auto-starts local Ollama if not specified)",
            default_value = "localhost:11434"
        )]
        ollama_host: String,

        /// Maximum number of functions to analyze with AI (to control costs)
        #[arg(
            long = "ai-max-functions",
            help = "Maximum number of functions to analyze with AI",
            default_value = "20"
        )]
        ai_max_functions: usize,
    },

    /// Start the Candle LLM server for hosting models
    Serve {
        /// Configuration file path
        #[arg(
            long = "config",
            help = "Path to configuration file (default: candle-server.toml)"
        )]
        config_file: Option<PathBuf>,

        /// Server host address
        #[arg(
            long = "host",
            help = "Host address for the Candle server",
            default_value = "127.0.0.1"
        )]
        host: String,

        /// Server port
        #[arg(
            long = "port",
            help = "Port for the Candle server",
            default_value = "8080"
        )]
        port: u16,

        /// AI model to use
        #[arg(long = "ai-model", help = "Specific AI model to use")]
        ai_model: Option<String>,
    },

    /// Download a model for the Candle server
    DownloadModel {
        /// Model ID to download
        #[arg(help = "Model ID to download (e.g., microsoft/DialoGPT-medium)")]
        model_id: String,
    },

    /// List available models and system compatibility
    ListModels,

    /// Generate default configuration file
    InitConfig {
        /// Configuration file path
        #[arg(
            long = "config",
            help = "Path to configuration file (default: candle-server.toml)"
        )]
        config_file: Option<PathBuf>,
    },
}

impl Args {
    /// Get import-related arguments (for compatibility with existing code)
    pub fn get_import_args(&self) -> Option<ImportArgs> {
        match &self.command {
            Commands::Imports {
                source_crate,
                target_crate,
                self_refactor,
                positional,
            } => Some(ImportArgs {
                source_crate: source_crate.clone(),
                target_crate: target_crate.clone(),
                self_refactor: *self_refactor,
                positional: positional.clone(),
                workspace_root: self.workspace_root.clone(),
                dry_run: self.dry_run,
                verbose: self.verbose,
            }),
            _ => None,
        }
    }

    /// Get analysis-related arguments
    pub fn get_analysis_args(&self) -> Option<AnalysisArgs> {
        match &self.command {
            Commands::Analyze {
                enable_ai,
                ai_provider,
                ai_model,
                ollama_host,
                ai_max_functions,
            } => Some(AnalysisArgs {
                enable_ai: *enable_ai,
                ai_provider: ai_provider.clone(),
                ai_model: ai_model.clone(),
                ollama_host: ollama_host.clone(),
                ai_max_functions: *ai_max_functions,
                workspace_root: self.workspace_root.clone(),
                dry_run: self.dry_run,
                verbose: self.verbose,
            }),
            _ => None,
        }
    }

    /// Get server-related arguments
    pub fn get_server_args(&self) -> Option<ServerArgs> {
        match &self.command {
            Commands::Serve {
                config_file,
                host,
                port,
                ai_model,
            } => Some(ServerArgs {
                config_file: config_file.clone(),
                host: host.clone(),
                port: *port,
                ai_model: ai_model.clone(),
                workspace_root: self.workspace_root.clone(),
                dry_run: self.dry_run,
                verbose: self.verbose,
            }),
            _ => None,
        }
    }
}

/// Import refactoring arguments (for compatibility)
#[derive(Debug)]
pub struct ImportArgs {
    pub source_crate: Option<String>,
    pub target_crate: Option<String>,
    pub self_refactor: bool,
    pub positional: Vec<String>,
    pub workspace_root: PathBuf,
    pub dry_run: bool,
    pub verbose: bool,
}

impl ImportArgs {
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

/// Analysis arguments
#[derive(Debug)]
pub struct AnalysisArgs {
    pub enable_ai: bool,
    pub ai_provider: String,
    pub ai_model: Option<String>,
    pub ollama_host: String,
    pub ai_max_functions: usize,
    pub workspace_root: PathBuf,
    pub dry_run: bool,
    pub verbose: bool,
}

/// Server arguments
#[derive(Debug)]
pub struct ServerArgs {
    pub config_file: Option<PathBuf>,
    pub host: String,
    pub port: u16,
    pub ai_model: Option<String>,
    pub workspace_root: PathBuf,
    pub dry_run: bool,
    pub verbose: bool,
}
