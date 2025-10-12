#[cfg(not(test))]
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    /// Workspace root directory
    #[arg(
        short = 'w',
        long,
        alias = "workspace",
        default_value = ".",
        global = true
    )]
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
    #[command(after_help = "EXAMPLES:
  refactor-tool imports core utils
  refactor-tool imports --source-crate core --target-crate utils  
  refactor-tool imports --self my_crate")]
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

        /// Keep super:: imports as-is instead of normalizing them to crate:: format
        #[arg(
            long = "keep-super",
            help = "Disable super:: imports normalization (default: normalize super:: to crate:: format). Can be used as --keep-super, --keep-super=true, or --keep-super=false",
            action = clap::ArgAction::Set,
            value_name = "BOOL",
            num_args = 0..=1,
            default_missing_value = "true",
            require_equals = false
        )]
        keep_super: Option<bool>,

        /// Keep exports unmodified (disable export generation)
        #[arg(
            long = "keep-exports",
            help = "Keep exports unmodified and disable automatic generation of pub use statements (default: false, exports are generated). Can be used as --keep-exports, --keep-exports=true, or --keep-exports=false",
            action = clap::ArgAction::Set,
            value_name = "BOOL", 
            num_args = 0..=1,
            default_missing_value = "true",
            require_equals = false
        )]
        keep_exports: Option<bool>,

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
                keep_super,
                keep_exports,
                positional,
            } => Some(ImportArgs {
                source_crate: source_crate.clone(),
                target_crate: target_crate.clone(),
                self_refactor: *self_refactor,
                keep_super: keep_super.unwrap_or(false),
                keep_exports: keep_exports.unwrap_or(false), // Default to false (enable export generation)
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
    pub keep_super: bool,
    pub keep_exports: bool,
    pub positional: Vec<String>,
    pub workspace_root: PathBuf,
    pub dry_run: bool,
    pub verbose: bool,
}

impl ImportArgs {
    /// Infer workspace root from crate paths if not explicitly provided
    #[cfg(not(test))]
    pub fn get_workspace_root(&self) -> Result<PathBuf> {
        // If workspace was explicitly set (not the default "."), use it
        let current_dir = std::env::current_dir()?;
        let workspace_canonical = self
            .workspace_root
            .canonicalize()
            .unwrap_or_else(|_| self.workspace_root.clone());
        let current_canonical =
            current_dir.canonicalize().unwrap_or(current_dir);

        let workspace_is_default = workspace_canonical == current_canonical;

        if !workspace_is_default {
            return Ok(self.workspace_root.clone());
        }

        // Try to infer workspace from crate paths
        if self.self_refactor {
            // Self-refactor mode: infer from single crate path
            let crate_name = self.get_self_crate()?;
            self.infer_workspace_from_crate_path(&crate_name)
        } else {
            // Cross-crate mode: infer from both crate paths and validate they're in same workspace
            let source_crate = self.get_source_crate()?;
            let target_crate = self.get_target_crate()?;

            let source_workspace =
                self.infer_workspace_from_crate_path(&source_crate)?;
            let target_workspace =
                self.infer_workspace_from_crate_path(&target_crate)?;

            // Check if both crates are in the same workspace
            let source_canonical = source_workspace
                .canonicalize()
                .unwrap_or_else(|_| source_workspace.clone());
            let target_canonical = target_workspace
                .canonicalize()
                .unwrap_or_else(|_| target_workspace.clone());

            if source_canonical != target_canonical {
                return Err(anyhow::anyhow!(
                    "Configuration error: Crates are in different workspaces.\n\
                     Source crate '{}' workspace: {}\n\
                     Target crate '{}' workspace: {}\n\
                     Please specify a common workspace root using --workspace-root or --workspace.",
                    source_crate, source_canonical.display(),
                    target_crate, target_canonical.display()
                ));
            }

            Ok(source_workspace)
        }
    }

    /// Infer workspace root from a single crate path
    #[cfg(not(test))]
    fn infer_workspace_from_crate_path(
        &self,
        crate_name: &str,
    ) -> Result<PathBuf> {
        let crate_path = PathBuf::from(crate_name);

        // If it's just a name (no path separators), assume current directory
        if crate_path.components().count() == 1 {
            return Ok(PathBuf::from("."));
        }

        // If it's a relative path, get the parent directory as workspace
        if crate_path.is_relative() {
            if let Some(parent) = crate_path.parent() {
                Ok(parent.to_path_buf())
            } else {
                Ok(PathBuf::from("."))
            }
        } else {
            // For absolute paths, we can't easily infer workspace, use current directory
            Ok(PathBuf::from("."))
        }
    }

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

#[cfg(test)]
impl ImportArgs {
    /// Test version of get_workspace_root
    pub fn get_workspace_root(&self) -> anyhow::Result<PathBuf> {
        // For tests, just return the configured workspace_root
        Ok(self.workspace_root.clone())
    }

    /// Test version of get_source_crate
    pub fn get_source_crate(&self) -> anyhow::Result<String> {
        if let Some(source) = &self.source_crate {
            Ok(source.clone())
        } else if !self.positional.is_empty() {
            Ok(self.positional[0].clone())
        } else {
            Err(anyhow::anyhow!("Source crate must be specified"))
        }
    }

    /// Test version of get_target_crate
    pub fn get_target_crate(&self) -> anyhow::Result<String> {
        if let Some(target) = &self.target_crate {
            Ok(target.clone())
        } else if self.positional.len() >= 2 {
            Ok(self.positional[1].clone())
        } else {
            Err(anyhow::anyhow!("Target crate must be specified"))
        }
    }

    /// Test version of get_self_crate
    pub fn get_self_crate(&self) -> anyhow::Result<String> {
        if let Some(source) = &self.source_crate {
            Ok(source.clone())
        } else if !self.positional.is_empty() {
            Ok(self.positional[0].clone())
        } else {
            Err(anyhow::anyhow!("Crate must be specified when using --self"))
        }
    }

    /// Test version of infer_workspace_from_crate_path
    pub fn infer_workspace_from_crate_path(
        &self,
        crate_name: &str,
    ) -> anyhow::Result<PathBuf> {
        let crate_path = PathBuf::from(crate_name);

        // If it's just a name (no path separators), assume current directory
        if crate_path.components().count() == 1 {
            return Ok(PathBuf::from("."));
        }

        // If it's a relative path, get the parent directory as workspace
        if crate_path.is_relative() {
            if let Some(parent) = crate_path.parent() {
                Ok(parent.to_path_buf())
            } else {
                Ok(PathBuf::from("."))
            }
        } else {
            // For absolute paths, we can't easily infer workspace, use current directory
            Ok(PathBuf::from("."))
        }
    }

    /// Test version of get_workspace_root with full inference logic
    pub fn get_workspace_root_with_inference(&self) -> anyhow::Result<PathBuf> {
        // If workspace was explicitly set (not the default "."), use it
        let workspace_is_default = self.workspace_root == PathBuf::from(".");

        if !workspace_is_default {
            return Ok(self.workspace_root.clone());
        }

        // Try to infer workspace from crate paths
        if self.self_refactor {
            // Self-refactor mode: infer from single crate path
            let crate_name = self.get_self_crate()?;
            self.infer_workspace_from_crate_path(&crate_name)
        } else {
            // Cross-crate mode: infer from both crate paths and validate they're in same workspace
            let source_crate = self.get_source_crate()?;
            let target_crate = self.get_target_crate()?;

            let source_workspace =
                self.infer_workspace_from_crate_path(&source_crate)?;
            let target_workspace =
                self.infer_workspace_from_crate_path(&target_crate)?;

            // Check if both crates are in the same workspace
            if source_workspace != target_workspace {
                return Err(anyhow::anyhow!(
                    "Configuration error: Crates are in different workspaces.\n\
                     Source crate '{}' workspace: {}\n\
                     Target crate '{}' workspace: {}\n\
                     Please specify a common workspace root using --workspace-root or --workspace.",
                    source_crate, source_workspace.display(),
                    target_crate, target_workspace.display()
                ));
            }

            Ok(source_workspace)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    fn create_test_import_args(
        source_crate: Option<String>,
        target_crate: Option<String>,
        positional: Vec<String>,
        workspace_root: PathBuf,
        self_refactor: bool,
    ) -> ImportArgs {
        ImportArgs {
            source_crate,
            target_crate,
            self_refactor,
            keep_super: false,
            keep_exports: false,
            positional,
            workspace_root,
            dry_run: false,
            verbose: false,
        }
    }

    #[test]
    fn test_workspace_inference_from_relative_path() {
        use std::path::PathBuf;

        // Test case: crate with relative path should infer workspace
        let args = ImportArgs {
            source_crate: None,
            target_crate: None,
            self_refactor: true,
            keep_super: false,
            keep_exports: false,
            positional: vec!["workspace/my_crate".to_string()],
            workspace_root: PathBuf::from("."), // default workspace
            dry_run: false,
            verbose: false,
        };

        // Mock the workspace inference for testing
        let workspace = args
            .infer_workspace_from_crate_path("workspace/my_crate")
            .unwrap();
        assert_eq!(workspace, PathBuf::from("workspace"));
    }

    #[test]
    fn test_workspace_inference_single_name() {
        // Test case: just crate name should use current directory
        let workspace = ImportArgs {
            source_crate: None,
            target_crate: None,
            self_refactor: true,
            keep_super: false,
            keep_exports: false,
            positional: vec!["my_crate".to_string()],
            workspace_root: PathBuf::from("."),
            dry_run: false,
            verbose: false,
        }
        .infer_workspace_from_crate_path("my_crate")
        .unwrap();

        assert_eq!(workspace, PathBuf::from("."));
    }

    #[test]
    fn test_cross_crate_same_workspace() {
        // Test that cross-crate with same workspace works
        let args = ImportArgs {
            source_crate: None,
            target_crate: None,
            self_refactor: false,
            keep_super: false,
            keep_exports: false,
            positional: vec![
                "workspace/crate1".to_string(),
                "workspace/crate2".to_string(),
            ],
            workspace_root: PathBuf::from("."),
            dry_run: false,
            verbose: false,
        };

        let workspace1 = args
            .infer_workspace_from_crate_path("workspace/crate1")
            .unwrap();
        let workspace2 = args
            .infer_workspace_from_crate_path("workspace/crate2")
            .unwrap();

        assert_eq!(workspace1, workspace2);
        assert_eq!(workspace1, PathBuf::from("workspace"));
    }

    #[test]
    fn test_cross_crate_different_workspaces_error() {
        // Test that cross-crate with different workspaces throws error
        let args = ImportArgs {
            source_crate: None,
            target_crate: None,
            self_refactor: false,
            keep_super: false,
            keep_exports: false,
            positional: vec![
                "workspace1/crate1".to_string(),
                "workspace2/crate2".to_string(),
            ],
            workspace_root: PathBuf::from("."), // default workspace
            dry_run: false,
            verbose: false,
        };

        let result = args.get_workspace_root_with_inference();
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains(
            "Configuration error: Crates are in different workspaces"
        ));
        assert!(error_msg.contains("workspace1"));
        assert!(error_msg.contains("workspace2"));
    }

    #[test]
    fn test_explicit_workspace_takes_precedence() {
        // Test that explicit workspace overrides inference
        let args = ImportArgs {
            source_crate: None,
            target_crate: None,
            self_refactor: false,
            keep_super: false,
            keep_exports: false,
            positional: vec![
                "workspace1/crate1".to_string(),
                "workspace2/crate2".to_string(),
            ],
            workspace_root: PathBuf::from("/explicit/workspace"), // explicit workspace
            dry_run: false,
            verbose: false,
        };

        let result = args.get_workspace_root_with_inference().unwrap();
        assert_eq!(result, PathBuf::from("/explicit/workspace"));
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
