// Candle Server Binary - Start the local LLM server
// This binary starts the candle-based LLM server for local model hosting

use anyhow::{
    Context,
    Result,
};
use clap::{
    Arg,
    Command,
};
use refactor_tool::server::config::ServerConfig;
use std::path::PathBuf;

#[cfg(feature = "embedded-llm")]
use refactor_tool::server::candle::CandleServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    let matches = Command::new("candle-server")
        .version("0.1.0")
        .about("Local LLM server using Candle for fast inference")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path")
        )
        .arg(
            Arg::new("host")
                .long("host")
                .value_name("HOST")
                .default_value("127.0.0.1")
                .help("Server host address")
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .default_value("8080")
                .help("Server port")
        )
        .arg(
            Arg::new("model")
                .short('m')
                .long("model")
                .value_name("MODEL_ID")
                .help("Hugging Face model ID (e.g., microsoft/CodeLlama-7b-Instruct-hf)")
        )
        .arg(
            Arg::new("device")
                .short('d')
                .long("device")
                .value_name("DEVICE")
                .default_value("auto")
                .help("Compute device: cpu, cuda, metal, auto")
        )
        .arg(
            Arg::new("download-only")
                .long("download-only")
                .action(clap::ArgAction::SetTrue)
                .help("Download model and exit (don't start server)")
        )
        .get_matches();

    #[cfg(not(feature = "embedded-llm"))]
    {
        eprintln!("❌ Embedded LLM feature not compiled.");
        eprintln!(
            "   Rebuild with: cargo build --release --features embedded-llm"
        );
        std::process::exit(1);
    }

    #[cfg(feature = "embedded-llm")]
    {
        run_server(matches).await
    }
}

#[cfg(feature = "embedded-llm")]
async fn run_server(matches: clap::ArgMatches) -> Result<()> {
    println!("🚀 Starting Candle LLM Server...");

    // Load configuration
    let config_path = matches
        .get_one::<String>("config")
        .map(|s| PathBuf::from(s));

    let mut config = ServerConfig::load_or_default(config_path.as_ref())
        .context("Failed to load configuration")?;

    // Override with command line arguments
    if let Some(host) = matches.get_one::<String>("host") {
        config.host = host.clone();
    }
    if let Some(port_str) = matches.get_one::<String>("port") {
        config.port = port_str.parse().context("Invalid port number")?;
    }
    if let Some(model) = matches.get_one::<String>("model") {
        config.model.model_id = model.clone();
    }
    if let Some(device) = matches.get_one::<String>("device") {
        config.model.device = device.clone();
    }

    // Validate configuration
    config
        .validate()
        .context("Configuration validation failed")?;

    println!("📋 Configuration:");
    println!("   Host: {}", config.host);
    println!("   Port: {}", config.port);
    println!("   Model: {}", config.model.model_id);
    println!("   Device: {}", config.model.device);
    println!("   Cache: {:?}", config.cache.cache_dir);

    // Create server instance
    println!("🤖 Initializing model...");
    let server = match CandleServer::with_config(config)
        .await
        .context("Failed to create server")?
    {
        Some(server) => server,
        None => {
            // User chose to quit gracefully
            println!("✅ Exited gracefully");
            return Ok(());
        },
    };

    // If download-only mode, exit after model is ready
    if matches.get_flag("download-only") {
        println!("✅ Model downloaded and validated successfully!");
        println!("   Use the same command without --download-only to start the server.");
        return Ok(());
    }

    // Start the HTTP server
    println!("🌐 Starting HTTP server...");
    let config = server.get_config().await;
    println!(
        "   OpenAI-compatible API at: http://{}:{}/v1/chat/completions",
        config.host, config.port
    );
    println!(
        "   Health check at: http://{}:{}/health",
        config.host, config.port
    );
    println!(
        "   Model info at: http://{}:{}/v1/models",
        config.host, config.port
    );
    println!();
    println!("Press Ctrl+C to stop the server");

    // Handle graceful shutdown
    let server_task = tokio::spawn(async move {
        if let Err(e) = server.start_server().await {
            eprintln!("Server error: {}", e);
        }
    });

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;
    println!("\n🛑 Shutdown signal received...");

    // Cancel the server task
    server_task.abort();

    println!("✅ Server stopped gracefully");
    Ok(())
}

#[cfg(not(feature = "embedded-llm"))]
async fn run_server(_matches: clap::ArgMatches) -> Result<()> {
    unreachable!("This function should not be called when embedded-llm feature is disabled");
}
