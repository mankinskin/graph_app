use anyhow::Result;
use std::path::PathBuf;

use crate::{
    core::{RefactorApi, RefactorConfigBuilder},
    analysis::crates::CrateNames,
    cli::args::Args,
};

pub fn run_refactor(args: &Args) -> Result<()> {
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

#[cfg(feature = "ai")]
pub async fn run_analysis(args: &Args) -> Result<()> {
    use crate::analysis::duplication::AiProvider;

    let ai_provider = match args.ai_provider.to_lowercase().as_str() {
        "openai" => AiProvider::OpenAI,
        "claude" => AiProvider::Claude,
        "ollama" => AiProvider::Ollama,
        "embedded" => AiProvider::Embedded,
        "auto" => AiProvider::Auto,
        _ =>
            return Err(anyhow::anyhow!(
                "Invalid AI provider. Use: openai, claude, ollama, embedded, or auto"
            )),
    };

    // Implementation would use the duplication analyzer
    println!("Analysis functionality temporarily disabled during refactoring");
    Ok(())
}

#[cfg(not(feature = "ai"))]
pub async fn run_analysis(_args: &Args) -> Result<()> {
    Err(anyhow::anyhow!(
        "Analysis features not available. Rebuild with --features ai to enable."
    ))
}

#[cfg(feature = "embedded-llm")]
pub async fn run_server(args: &Args) -> Result<()> {
    use crate::server::{ServerConfig, CandleServer};

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
pub async fn run_server(_args: &Args) -> Result<()> {
    Err(anyhow::anyhow!(
        "Embedded LLM server not available. Rebuild with --features embedded-llm to enable."
    ))
}

#[cfg(feature = "embedded-llm")]
pub async fn download_model(args: &Args, model_id: &str) -> Result<()> {
    use crate::server::{ServerConfig, CandleServer};

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
pub async fn download_model(_args: &Args, model_id: &str) -> Result<()> {
    Err(anyhow::anyhow!(
        "Model download not available. Rebuild with --features embedded-llm to enable downloading {}.", 
        model_id
    ))
}

#[cfg(feature = "embedded-llm")]
pub fn list_models() -> Result<()> {
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
pub fn list_models() -> Result<()> {
    Err(anyhow::anyhow!(
        "Model listing not available. Rebuild with --features embedded-llm to enable."
    ))
}

#[cfg(feature = "embedded-llm")]
pub fn init_config(args: &Args) -> Result<()> {
    use crate::server::ServerConfig;

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
pub fn init_config(_args: &Args) -> Result<()> {
    Err(anyhow::anyhow!(
        "Configuration initialization not available. Rebuild with --features embedded-llm to enable."
    ))
}