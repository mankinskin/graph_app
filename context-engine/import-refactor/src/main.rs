#![feature(iter_intersperse)]

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod crate_analyzer;
mod import_parser;
mod item_info;
mod refactor_api;
mod refactor_engine;
mod utils;

use refactor_api::{
    RefactorApi,
    RefactorConfigBuilder,
};

use crate::crate_analyzer::CrateNames;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
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

    /// Run duplication analyzer on the codebase
    #[arg(
        long = "analyze",
        help = "Analyze the codebase for duplicate and similar functions"
    )]
    analyze: bool,

    /// Start the Candle LLM server
    #[arg(
        long = "serve",
        help = "Start the local Candle LLM server for hosting models"
    )]
    serve: bool,

    /// Download model for the Candle server
    #[arg(
        long = "download-model",
        help = "Download a model for the Candle server"
    )]
    download_model: Option<String>,

    /// List available models and system compatibility
    #[arg(
        long = "list-models",
        help = "List recommended models and check system compatibility"
    )]
    list_models: bool,

    /// Generate default configuration file
    #[arg(
        long = "init-config",
        help = "Generate a default candle-server.toml configuration file"
    )]
    init_config: bool,

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

    /// Positional arguments: [SOURCE_CRATE] [TARGET_CRATE] or [CRATE] when using --self
    #[arg(
        help = "Positional arguments: [SOURCE_CRATE] [TARGET_CRATE] or [CRATE] when using --self"
    )]
    positional: Vec<String>,

    /// Workspace root directory
    #[arg(short = 'w', long, default_value = ".")]
    workspace_root: PathBuf,

    /// Dry run - show what would be changed without modifying files
    #[arg(long)]
    dry_run: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

impl Args {
    /// Get the source crate name, preferring the flag over the positional argument
    fn get_source_crate(&self) -> Result<String> {
        if let Some(source) = &self.source_crate {
            Ok(source.clone())
        } else if !self.positional.is_empty() {
            Ok(self.positional[0].clone())
        } else {
            Err(anyhow::anyhow!("Source crate must be specified either via --source-crate/--source flag or as the first positional argument"))
        }
    }

    /// Get the target crate name, preferring the flag over the positional argument
    fn get_target_crate(&self) -> Result<String> {
        if let Some(target) = &self.target_crate {
            Ok(target.clone())
        } else if self.positional.len() >= 2 {
            Ok(self.positional[1].clone())
        } else {
            Err(anyhow::anyhow!("Target crate must be specified either via --target-crate/--target flag or as the second positional argument"))
        }
    }

    /// Get the crate name for self-refactor mode
    fn get_self_crate(&self) -> Result<String> {
        if let Some(source) = &self.source_crate {
            Ok(source.clone())
        } else if !self.positional.is_empty() {
            Ok(self.positional[0].clone())
        } else {
            Err(anyhow::anyhow!("Crate must be specified either via --source-crate/--source flag or as the first positional argument when using --self"))
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.serve {
        // Server mode: start the Candle LLM server
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(run_server(&args))
    } else if let Some(model_id) = &args.download_model {
        // Download model mode
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(download_model(&args, model_id))
    } else if args.list_models {
        // List models mode
        list_models()
    } else if args.init_config {
        // Initialize configuration
        init_config(&args)
    } else if args.analyze {
        // Analyzer mode: analyze codebase for duplications (async for AI support)
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(run_analyzer(&args))
    } else {
        // Import refactor mode (handles both self-refactor and standard modes)
        run_import_refactor(&args)
    }
}

fn run_import_refactor(args: &Args) -> Result<()> {
    let workspace_root = args
        .workspace_root
        .canonicalize()
        .unwrap_or_else(|_| args.workspace_root.clone());

    let crate_names = if args.self_refactor {
        let crate_name = args.get_self_crate()?;
        CrateNames::SelfRefactor { crate_name }
    } else {
        let source_crate = args.get_source_crate()?;
        let target_crate = args.get_target_crate()?;
        CrateNames::CrossRefactor {
            source_crate,
            target_crate,
        }
    };

    let config = RefactorConfigBuilder::new()
        .crate_names(crate_names)
        .workspace_root(workspace_root)
        .dry_run(args.dry_run)
        .verbose(args.verbose)
        .quiet(false)
        .build()?;

    let result = RefactorApi::execute_refactor(config);

    if !result.success {
        if let Some(error) = result.error {
            return Err(error);
        } else {
            return Err(anyhow::anyhow!(
                "Refactoring failed for unknown reason"
            ));
        }
    }

    Ok(())
}

async fn run_analyzer(args: &Args) -> Result<()> {
    let ai_provider = match args.ai_provider.to_lowercase().as_str() {
        "openai" => utils::duplication_analyzer::AiProvider::OpenAI,
        "claude" => utils::duplication_analyzer::AiProvider::Claude,
        "ollama" => utils::duplication_analyzer::AiProvider::Ollama,
        "embedded" => utils::duplication_analyzer::AiProvider::Embedded,
        "auto" => utils::duplication_analyzer::AiProvider::Auto,
        _ =>
            return Err(anyhow::anyhow!(
                "Invalid AI provider. Use: openai, claude, ollama, embedded, or auto"
            )),
    };

    utils::analyzer_cli::run_analyzer_with_ai(
        Some(args.workspace_root.clone()),
        args.verbose,
        args.enable_ai,
        ai_provider,
        args.ai_model.clone(),
        Some(args.ollama_host.clone()),
        args.ai_max_functions,
    )
    .await
    .map_err(|e| anyhow::anyhow!("Analyzer failed: {}", e))
}

#[cfg(feature = "embedded-llm")]
async fn run_server(args: &Args) -> Result<()> {
    use utils::{
        candle_config::ServerConfig,
        candle_server::CandleServer,
    };

    println!("🚀 Starting Candle LLM Server");

    // Load configuration
    let config_path = args
        .config_file
        .as_ref()
        .cloned()
        .or_else(|| Some(PathBuf::from("candle-server.toml")));

    let server_config = ServerConfig::load_or_default(config_path.as_ref())?;
    server_config.validate()?;

    println!("📋 Configuration:");
    println!("   Host: {}", server_config.host);
    println!("   Port: {}", server_config.port);
    println!("   Model: {}", server_config.model.model_id);
    println!("   Device: {}", server_config.model.device);

    // Create and start the server
    let server = match CandleServer::with_config(server_config).await? {
        Some(server) => server,
        None => {
            // User chose to quit gracefully
            println!("✅ Exited gracefully");
            return Ok(());
        }
    };

    println!("🌐 Server starting...");
    println!("   Ctrl+C to stop");

    server.start_server().await?;

    Ok(())
}

#[cfg(not(feature = "embedded-llm"))]
async fn run_server(_args: &Args) -> Result<()> {
    Err(anyhow::anyhow!(
        "Embedded LLM server not available. Rebuild with --features embedded-llm to enable."
    ))
}

#[cfg(feature = "embedded-llm")]
async fn download_model(
    args: &Args,
    model_id: &str,
) -> Result<()> {
    use utils::{
        candle_config::ServerConfig,
        candle_server::CandleServer,
    };

    println!("📥 Downloading model: {}", model_id);

    // Create a config with the specified model
    let mut config = ServerConfig::default();
    config.model.model_id = model_id.to_string();

    // Create server instance which will download the model
    match CandleServer::with_config(config).await? {
        Some(_server) => {
            println!("✅ Model downloaded successfully!");
        }
        None => {
            println!("❌ Download cancelled by user");
            return Ok(());
        }
    }

    Ok(())
}

#[cfg(not(feature = "embedded-llm"))]
async fn download_model(
    _args: &Args,
    model_id: &str,
) -> Result<()> {
    Err(anyhow::anyhow!(
        "Model download not available. Rebuild with --features embedded-llm to enable downloading {}.", 
        model_id
    ))
}

#[cfg(feature = "embedded-llm")]
fn list_models() -> Result<()> {
    println!("📋 Candle LLM Models");
    println!("===================");

    println!("\n🖥️  System Information:");
    println!(
        "   CUDA: {}",
        if candle_core::Device::cuda_if_available(0).is_ok() {
            "✅ Available"
        } else {
            "❌ Not available"
        }
    );
    println!(
        "   Metal: {}",
        if candle_core::Device::new_metal(0).is_ok() {
            "✅ Available"
        } else {
            "❌ Not available"
        }
    );

    println!("\n📚 Model Support:");
    println!(
        "   • Any Hugging Face model compatible with the Llama architecture"
    );
    println!("   • Specify model ID when downloading or configuring");
    println!("   • Popular choices: CodeLlama, Qwen2.5-Coder, DeepSeek-Coder");

    println!("\n🚀 Usage:");
    println!("   import-refactor --download-model <model-id>");
    println!("   import-refactor --serve --ai-model <model-id>");

    Ok(())
}

#[cfg(not(feature = "embedded-llm"))]
fn list_models() -> Result<()> {
    Err(anyhow::anyhow!(
        "Model listing not available. Rebuild with --features embedded-llm to enable."
    ))
}

#[cfg(feature = "embedded-llm")]
fn init_config(args: &Args) -> Result<()> {
    use utils::candle_config::ServerConfig;

    let config_path = args
        .config_file
        .as_ref()
        .cloned()
        .unwrap_or_else(|| PathBuf::from("candle-server.toml"));

    if config_path.exists() {
        println!("⚠️  Configuration file already exists: {:?}", config_path);
        println!("❓ Overwrite? (y/N): ");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().to_lowercase().starts_with('y') {
            println!("❌ Configuration initialization cancelled.");
            return Ok(());
        }
    }

    let config = ServerConfig::default();
    config.save(&config_path)?;

    println!("✅ Configuration file created: {:?}", config_path);
    println!(
        "📝 You can now edit this file to customize your server settings."
    );
    println!("\n🔧 Key settings you might want to adjust:");
    println!("   • model.model_id - Choose from the available models");
    println!("   • model.device - Set to 'cuda', 'metal', or 'cpu'");
    println!("   • server.host and server.port - Network settings");
    println!("   • cache.cache_dir - Where to store downloaded models");

    println!("\n📋 To see available models, run:");
    println!("   import-refactor --list-models");

    Ok(())
}

#[cfg(not(feature = "embedded-llm"))]
fn init_config(_args: &Args) -> Result<()> {
    Err(anyhow::anyhow!(
        "Configuration initialization not available. Rebuild with --features embedded-llm to enable."
    ))
}
